use std::{collections::HashMap, time::Duration};

use chrono::Timelike;

mod ggtask_client;

const CLIENT_1_TASK_LIST_NAME: &str = "My Tasks";
const CLIENT_2_TASK_LIST_NAME: &str = "TODO";
const SHARE_PREFIX: &str = "share";

#[tokio::main]
async fn main() {
    let client_cfg_file = std::env::var("CLIENT_CONFIG_FILE").unwrap();

    let t_client1 = ggtask_client::Client::new(&client_cfg_file, "token-1.json")
        .await
        .unwrap();
    let t_client2 = ggtask_client::Client::new(&client_cfg_file, "token-2.json")
        .await
        .unwrap();

    execution(&t_client1, &t_client2).await.unwrap();
}

async fn execution(c1: &ggtask_client::Client, c2: &ggtask_client::Client) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(60));

    loop {
        let hm_tasks1 = match build_tasks_map(c1, CLIENT_1_TASK_LIST_NAME, SHARE_PREFIX).await {
            Ok(m) => m,
            Err(_) => {
                let task_lists = c1.list_task_lists().await?;
                let list_id =
                    ggtask_client::find_list_id_by_title(&task_lists, CLIENT_1_TASK_LIST_NAME)?;
                (list_id, HashMap::new())
            }
        };

        let hm_tasks2 = match build_tasks_map(c2, CLIENT_2_TASK_LIST_NAME, SHARE_PREFIX).await {
            Ok(m) => m,
            Err(_) => {
                let task_lists = c2.list_task_lists().await?;
                let list_id =
                    ggtask_client::find_list_id_by_title(&task_lists, CLIENT_2_TASK_LIST_NAME)?;
                (list_id, HashMap::new())
            }
        };

        let act1 = match merge_tasks(&hm_tasks1.1, &hm_tasks2.1) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("merge tasks 1 <- 2 error {e}");
                continue;
            }
        };

        match sync_tasks(c1, &hm_tasks1.0, act1).await {
            Ok(()) => {
                println!("sync task c1 ok");
            }
            Err(e) => {
                eprintln!("sync task c1 error: {e}")
            }
        }

        let act2 = match merge_tasks(&hm_tasks2.1, &hm_tasks1.1) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("merge tasks 2 <- 1 error {e}");
                continue;
            }
        };

        match sync_tasks(c2, &hm_tasks2.0, act2).await {
            Ok(()) => {
                println!("sync task c2 ok");
            }
            Err(e) => {
                eprintln!("sync task c2 error: {e}")
            }
        }

        println!("wait");
        interval.tick().await;
    }
}

// sync_tasks
// actions: actions need to do to with c.
async fn sync_tasks(
    c: &ggtask_client::Client,
    task_list_id: &str,
    actions: Vec<(google_tasks1::api::Task, Action)>,
) -> anyhow::Result<()> {
    for (t, act) in actions.iter() {
        match act {
            Action::Add => match c
                .create_task(
                    task_list_id,
                    t.title.as_ref().unwrap(),
                    t.due.as_ref().unwrap(),
                )
                .await
            {
                Ok(()) => {
                    println!("new task created");
                }
                Err(e) => {
                    eprintln!("create task error {e}");
                }
            },
            Action::Complete => {
                match c
                    .complete_task(
                        task_list_id,
                        t.id.as_ref().unwrap(),
                        t.completed.as_ref().unwrap(),
                    )
                    .await
                {
                    Ok(()) => {
                        println!("completed task")
                    }
                    Err(e) => {
                        eprintln!("complete task error {e}")
                    }
                }
            }
            _ => {
                println!("action not supported {:#?}", act)
            }
        }
    }

    Ok(())
}

async fn build_tasks_map(
    c: &ggtask_client::Client,
    list: &str,
    prefix: &str,
) -> anyhow::Result<(String, HashMap<String, google_tasks1::api::Task>)> {
    // get the list id
    let task_lists = c.list_task_lists().await?;
    let list_id = ggtask_client::find_list_id_by_title(&task_lists, list)?;

    // get tasks
    let now = chrono::Utc::now();
    let seven_days_before = now
        .checked_sub_days(chrono::Days::new(7))
        .unwrap()
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();

    let tasks = c
        .list_tasks(&list_id, Some(seven_days_before.to_rfc3339().as_str()))
        .await?;

    let mut hm = HashMap::<String, google_tasks1::api::Task>::new();

    for t in tasks {
        let title = match t.title.as_ref() {
            None => continue,
            Some(ti) => ti,
        };

        match get_prefix(title) {
            None => continue,
            Some(pre) => {
                if !pre.eq(prefix) {
                    continue;
                }
            }
        }

        hm.insert(title.to_owned(), t);
    }

    Ok((list_id, hm))
}

fn get_prefix(title: &str) -> Option<String> {
    // [<prefix>] task name
    if title.len() < 3 {
        return None;
    }

    if !title.starts_with('[') {
        return None;
    }

    let prefix_end = match title.find(']') {
        None => return None,
        Some(i) => i,
    };

    let prefix = String::from(&title[1..prefix_end]);

    Some(prefix)
}

#[derive(Debug, Clone, PartialEq)]
enum Action {
    Add,
    Delete,
    Complete,
    Uncomplete,
}

// m2 = src, m1 = dest. Update the m1 to have the same task as m2
fn merge_tasks(
    m1: &HashMap<String, google_tasks1::api::Task>,
    m2: &HashMap<String, google_tasks1::api::Task>,
) -> anyhow::Result<Vec<(google_tasks1::api::Task, Action)>> {
    let mut v1: Vec<(google_tasks1::api::Task, Action)> = Vec::new();
    for (t2_title, t2) in m2.iter() {
        println!("working {}", t2_title);
        if let Some(deleted) = t2.deleted {
            if deleted {
                continue;
            }
        }

        let t1 = match m1.get(t2_title) {
            None => {
                println!("new task {:#?}", t2);
                v1.push((
                    google_tasks1::api::Task {
                        completed: t2.completed.clone(),
                        deleted: None,
                        due: t2.due.clone(),
                        etag: None,
                        hidden: None,
                        id: None,
                        kind: None,
                        links: None,
                        notes: None,
                        parent: None,
                        position: None,
                        self_link: None,
                        status: t2.status.clone(),
                        title: t2.title.clone(),
                        updated: None,
                    },
                    Action::Add,
                ));
                continue;
            }
            Some(t) => t,
        };

        if t1.updated.is_none() || t2.updated.is_none() {
            continue;
        }

        let t1_updated = chrono::DateTime::parse_from_rfc3339(t1.updated.as_ref().unwrap())?;
        let t2_updated = chrono::DateTime::parse_from_rfc3339(t2.updated.as_ref().unwrap())?;

        if t2_updated.lt(&t1_updated) {
            continue;
        }

        if t2.deleted.is_some() && t1.deleted.is_none() {
            v1.push((t1.clone(), Action::Delete));
            continue;
        }

        if t2.completed.is_some() && t1.completed.is_none() {
            let mut t = t1.clone();
            t.completed = t2.completed.clone();
            v1.push((t, Action::Complete));
            continue;
        }

        if t2.completed.is_none() && t1.completed.is_some() {
            let mut t = t1.clone();
            t.completed = t2.completed.clone();
            v1.push((t, Action::Uncomplete));
            continue;
        }
    }

    Ok(v1)
}
