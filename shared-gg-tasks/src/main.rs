mod ggtask_client;

#[tokio::main]
async fn main() {
    let client_cfg_file = std::env::var("CLIENT_CONFIG_FILE").unwrap();

    let t_client = ggtask_client::Client::new(&client_cfg_file).await.unwrap();

    let task_lists = t_client.list_task_lists().await.unwrap();

    println!("{:#?}", task_lists);

    let list_id = ggtask_client::find_list_id_by_title(&task_lists, "TODO").unwrap();

    println!("{:#?}", list_id);

    let tasks = t_client.list_tasks(&list_id).await.unwrap();

    println!("{:#?}", tasks.len());

    tasks
        .iter()
        .for_each(|t| println!("{}", t.title.as_ref().unwrap()));
}
