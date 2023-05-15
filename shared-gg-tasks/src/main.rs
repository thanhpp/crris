use std::{collections::HashMap, str::FromStr};

use chrono::Timelike;

mod ggtask_client;

#[tokio::main]
async fn main() {
    let client_cfg_file = std::env::var("CLIENT_CONFIG_FILE").unwrap();

    let t_client1 = ggtask_client::Client::new(&client_cfg_file).await.unwrap();
    let hm_tasks1 = match build_tasks_map(&t_client1, "My Tasks", "vta").await {
        Ok(m) => m,
        Err(_) => {
            let task_lists = t_client1.list_task_lists().await.unwrap();
            let list_id = ggtask_client::find_list_id_by_title(&task_lists, "My Tasks").unwrap();
            (list_id, HashMap::new())
        }
    };

    let t_client2 = ggtask_client::Client::new(&client_cfg_file).await.unwrap();
    let hm_tasks2 = match build_tasks_map(&t_client2, "TODO", "vta").await {
        Ok(m) => m,
        Err(_) => {
            let task_lists = t_client2.list_task_lists().await.unwrap();
            let list_id = ggtask_client::find_list_id_by_title(&task_lists, "TODO").unwrap();
            (list_id, HashMap::new())
        }
    };

    let actions = match merge_tasks(&hm_tasks1.1, &hm_tasks2.1) {
        Ok(a) => a,
        Err(_) => Vec::new(),
    };

    for (t, act) in actions.iter() {
        match t_client1
            .create_task(
                &hm_tasks1.0,
                t.title.as_ref().unwrap(),
                t.due.as_ref().unwrap(),
            )
            .await
        {
            Ok(_) => {
                println!("created OK")
            }
            Err(e) => {
                println!("create task error {e}")
            }
        }
    }
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

    return Some(prefix);
}

#[derive(Debug, Clone, PartialEq)]
enum Action {
    Add,
    Update,
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

        if t2_updated.gt(&t1_updated) {
            continue;
        }

        if t2.deleted.is_none() && t1.deleted.is_some() {
            v1.push((t2.clone(), Action::Delete));
            continue;
        }

        if t2.completed.is_none() && t1.completed.is_some() {
            v1.push((t2.clone(), Action::Complete));
            continue;
        }

        if t2.completed.is_some() && t1.completed.is_none() {
            v1.push((t2.clone(), Action::Uncomplete));
            continue;
        }
    }

    Ok(v1)
}
