use std::fs;

use anyhow::{Ok, Result};
use google_tasks1::hyper_rustls::HttpsConnector;
use google_tasks1::{hyper, hyper_rustls};
use hyper::client::HttpConnector;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

pub struct Client {
    hub: google_tasks1::TasksHub<HttpsConnector<HttpConnector>>,
}

impl Client {
    pub async fn new(config_file: &str) -> Result<Client> {
        let f_data = fs::read_to_string(config_file)?;
        let client_cfg: ClientConfig = serde_json::from_str(&f_data)?;

        let secret = google_tasks1::oauth2::ApplicationSecret {
            client_id: client_cfg.installed.client_id,
            client_secret: client_cfg.installed.client_secret,
            token_uri: client_cfg.installed.token_uri,
            auth_uri: client_cfg.installed.auth_uri,
            redirect_uris: client_cfg.installed.redirect_uris,
            project_id: Some(client_cfg.installed.project_id),
            client_email: Some("thanhphanphu18@gmail.com".to_string()),
            auth_provider_x509_cert_url: Some(client_cfg.installed.auth_provider_x509_cert_url),
            client_x509_cert_url: None,
        };

        let auth = google_tasks1::oauth2::InstalledFlowAuthenticator::builder(
            secret,
            google_tasks1::oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .build()
        .await?;

        let hub = google_tasks1::TasksHub::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_or_http()
                    .enable_http1()
                    .enable_http2()
                    .build(),
            ),
            auth,
        );

        Ok(Client { hub })
    }

    pub async fn list_task_lists(&self) -> Result<google_tasks1::api::TaskLists> {
        let resp = self
            .hub
            .tasklists()
            .list()
            .add_scopes(vec![
                google_tasks1::api::Scope::Full,
                google_tasks1::api::Scope::Readonly,
            ])
            .doit()
            .await?;
        let resp_body = resp.0;

        if resp_body.status().ne(&StatusCode::OK) {
            return Err(anyhow::format_err!("not OK status: {}", resp_body.status()));
        }

        Ok(resp.1)
    }

    pub async fn list_tasks(
        &self,
        task_list_id: &str,
        due_min: Option<&str>,
    ) -> Result<Vec<google_tasks1::api::Task>> {
        let mut req = self.hub.tasks().list(task_list_id).max_results(100);

        if let Some(t) = due_min {
            req = req.due_min(t);
        }

        req = req.show_completed(true).show_hidden(true);

        let resp = req.doit().await?;

        Self::check_resp(&resp.0, "list tasks")?;

        let mut tasks = match resp.1.items {
            None => return Ok(vec![]),
            Some(t) => t,
        };

        let mut next_page_token = resp.1.next_page_token;
        while let Some(token) = next_page_token {
            let resp = self
                .hub
                .tasks()
                .list(task_list_id)
                .show_completed(true)
                .show_hidden(true)
                .page_token(&token)
                .max_results(100)
                .doit()
                .await?;

            Self::check_resp(&resp.0, "list tasks by page token")?;

            match resp.1.items {
                None => return Ok(tasks),
                Some(mut t) => tasks.append(&mut t),
            };

            next_page_token = resp.1.next_page_token
        }

        Ok(tasks)
    }

    pub async fn create_task(&self, task_list: &str, title: &str, due: &str) -> Result<()> {
        println!("creating {} {} {}", task_list, title, due);
        let resp = self
            .hub
            .tasks()
            .insert(
                google_tasks1::api::Task {
                    completed: None,
                    deleted: None,
                    due: Some(due.to_string()),
                    etag: None,
                    hidden: None,
                    id: None,
                    kind: None,
                    links: None,
                    notes: None,
                    parent: None,
                    position: None,
                    self_link: None,
                    status: None,
                    title: Some(title.to_string()),
                    updated: None,
                },
                task_list,
            )
            .doit()
            .await?;

        Self::check_resp(&resp.0, "create task")?;

        Ok(())
    }

    pub async fn complete_task(
        &self,
        task_list: &str,
        task_id: &str,
        completed: &str,
    ) -> Result<()> {
        println!("completing task {} {}", task_list, task_id);
        let resp = self
            .hub
            .tasks()
            .update(
                google_tasks1::api::Task {
                    completed: Some(String::from(completed)),
                    deleted: None,
                    due: None,
                    etag: None,
                    hidden: None,
                    id: None,
                    kind: None,
                    links: None,
                    notes: None,
                    parent: None,
                    position: None,
                    self_link: None,
                    status: None,
                    title: None,
                    updated: None,
                },
                task_list,
                task_id,
            )
            .doit()
            .await?;

        Self::check_resp(&resp.0, "complete task")?;

        Ok(())
    }

    fn check_resp(resp_body: &hyper::Response<hyper::body::Body>, op: &str) -> Result<()> {
        if resp_body.status().ne(&StatusCode::OK) {
            return Err(anyhow::format_err!(
                "{}: not OK status: {}",
                op,
                resp_body.status()
            ));
        }

        Ok(())
    }
}

pub fn find_list_id_by_title(
    task_lists: &google_tasks1::api::TaskLists,
    title: &str,
) -> Result<String> {
    let task_lists = match task_lists.items.as_ref() {
        None => return Err(anyhow::format_err!("empty task lists")),
        Some(l) => l,
    };

    for list in task_lists.iter() {
        match list.title.as_ref() {
            None => continue,
            Some(t) => {
                if t.eq(title) {
                    if let Some(id) = list.id.as_ref() {
                        return Ok(id.clone());
                    }
                }
            }
        }
    }

    Err(anyhow::format_err!("can not find list with given title"))
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientConfig {
    pub installed: Installed,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Installed {
    #[serde(rename = "client_id")]
    pub client_id: String,
    #[serde(rename = "project_id")]
    pub project_id: String,
    #[serde(rename = "auth_uri")]
    pub auth_uri: String,
    #[serde(rename = "token_uri")]
    pub token_uri: String,
    #[serde(rename = "auth_provider_x509_cert_url")]
    pub auth_provider_x509_cert_url: String,
    #[serde(rename = "client_secret")]
    pub client_secret: String,
    #[serde(rename = "redirect_uris")]
    pub redirect_uris: Vec<String>,
}
