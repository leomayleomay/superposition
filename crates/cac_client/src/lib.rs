mod eval;
mod interface;
mod utils;

use actix_web::{rt::time::interval, web::Data};
use chrono::{DateTime, Utc};
use derive_more::{Deref, DerefMut};
use reqwest::{RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    collections::HashMap,
    convert::identity,
    sync::{Arc, RwLock},
    time::{Duration, UNIX_EPOCH},
};
use strum_macros;
use utils::core::MapError;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Context {
    pub condition: Value,
    pub override_with_keys: [String; 1],
}

#[repr(C)]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    contexts: Vec<Context>,
    overrides: Map<String, Value>,
    default_configs: Map<String, Value>,
}

#[derive(strum_macros::EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MergeStrategy {
    MERGE,
    REPLACE,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        return Self::MERGE;
    }
}

impl From<String> for MergeStrategy {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "replace" => MergeStrategy::REPLACE,
            "merge" => MergeStrategy::MERGE,
            _ => MergeStrategy::default(),
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Client {
    tenant: String,
    reqw: Data<reqwest::RequestBuilder>,
    polling_interval: Duration,
    last_modified: Data<RwLock<DateTime<Utc>>>,
    config: Data<RwLock<Config>>,
}

fn clone_reqw(reqw: &RequestBuilder) -> Result<RequestBuilder, String> {
    reqw.try_clone()
        .ok_or_else(|| "Unable to clone reqw".to_string())
}

fn get_last_modified(resp: &Response) -> Option<DateTime<Utc>> {
    resp.headers().get("last-modified").and_then(|header_val| {
        let header_str = header_val.to_str().ok()?;
        DateTime::parse_from_rfc2822(header_str)
            .map(|datetime| datetime.with_timezone(&Utc))
            .map_err(|e| {
                log::error!("Failed to parse date: {e}");
            })
            .ok()
    })
}

impl Client {
    pub async fn new(
        tenant: String,
        polling_interval: Duration,
        hostname: String,
    ) -> Result<Self, String> {
        let reqw_client = reqwest::Client::builder().build().map_err_to_string()?;
        let cac_endpoint = format!("{hostname}/config");
        let reqw = reqw_client
            .get(cac_endpoint)
            .header("x-tenant", tenant.to_string());

        let reqwc = clone_reqw(&reqw)?;
        let resp = reqwc.send().await.map_err_to_string()?;
        let last_modified_at = get_last_modified(&resp);
        let config = resp.json::<Config>().await.map_err_to_string()?;

        let client = Client {
            tenant,
            reqw: Data::new(reqw),
            polling_interval,
            last_modified: Data::new(RwLock::new(
                last_modified_at.unwrap_or(DateTime::<Utc>::from(UNIX_EPOCH)),
            )),
            config: Data::new(RwLock::new(config)),
        };
        Ok(client)
    }

    async fn fetch(&self) -> Result<reqwest::Response, String> {
        let last_modified = self.last_modified.read().map_err_to_string()?.to_rfc2822();
        let reqw = clone_reqw(&self.reqw)?.header("If-Modified-Since", last_modified);
        let resp = reqw.send().await.map_err_to_string()?;
        match resp.status() {
            StatusCode::NOT_MODIFIED => {
                return Err(String::from(format!(
                    "{} CAC: skipping update, remote not modified",
                    self.tenant
                )));
            }
            StatusCode::OK => log::info!(
                "{}",
                format!("{} CAC: new config received, updating", self.tenant)
            ),
            x => return Err(format!("{} CAC: fetch failed, status: {}", self.tenant, x)),
        };
        Ok(resp)
    }

    async fn update_cac(&self) -> Result<String, String> {
        let fetched_config = self.fetch().await?;
        let mut config = self.config.write().map_err_to_string()?;
        let mut last_modified = self.last_modified.write().map_err_to_string()?;
        let last_modified_at = get_last_modified(&fetched_config);
        *config = fetched_config.json::<Config>().await.map_err_to_string()?;
        if let Some(val) = last_modified_at {
            *last_modified = val;
        }
        Ok(format!("{}: CAC updated successfully", self.tenant))
    }

    pub async fn run_polling_updates(self: Arc<Self>) {
        let mut interval = interval(self.polling_interval);
        loop {
            interval.tick().await;
            let result = self.update_cac().await.unwrap_or_else(identity);
            log::info!("{result}",);
        }
    }

    pub fn get_full_config_state(&self) -> Result<Config, String> {
        self.config.read().map(|c| c.clone()).map_err_to_string()
    }

    pub fn get_last_modified(&self) -> Result<DateTime<Utc>, String> {
        self.last_modified.read().map(|t| *t).map_err_to_string()
    }

    pub fn eval(
        &self,
        query_data: Map<String, Value>,
        merge_strategy: MergeStrategy,
    ) -> Result<Map<String, Value>, String> {
        let cac = self.config.read().map_err_to_string()?;
        eval::eval_cac(
            cac.default_configs.to_owned(),
            &cac.contexts,
            &cac.overrides,
            &query_data,
            merge_strategy,
        )
    }

    pub fn get_resolved_config(
        &self,
        query_data: Map<String, Value>,
        keys: Vec<String>,
        merge_strategy: MergeStrategy,
    ) -> Result<Map<String, Value>, String> {
        let cac = self.eval(query_data, merge_strategy)?;
        if keys.is_empty() {
            return Ok(cac);
        }
        Ok(Map::from_iter(
            cac.iter()
                .filter_map(|(config, v)| {
                    if keys.contains(config) {
                        Some((config.to_owned(), v.to_owned()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<(String, Value)>>(),
        ))
    }

    pub fn get_default_config(
        &self,
        keys: Vec<String>,
    ) -> Result<Map<String, Value>, String> {
        let configs = self.config.read().map_err(|e| e.to_string())?;
        let default_configs = configs.default_configs.clone();
        if keys.is_empty() {
            return Ok(default_configs);
        }
        let default_configs = default_configs
            .into_iter()
            .filter(|(item, _)| keys.contains(item))
            .collect::<Map<String, Value>>();
        Ok(default_configs)
    }
}

#[derive(Deref, DerefMut)]
pub struct ClientFactory(RwLock<HashMap<String, Arc<Client>>>);
impl ClientFactory {
    pub async fn create_client(
        &self,
        tenant: String,
        polling_interval: Duration,
        hostname: String,
    ) -> Result<Arc<Client>, String> {
        let mut factory = match self.write() {
            Ok(factory) => factory,
            Err(e) => {
                log::error!("CAC_CLIENT_FACTORY: failed to acquire write lock {}", e);
                return Err("CAC_CLIENT_FACTORY: Failed to create client".to_string());
            }
        };

        if let Some(client) = factory.get(&tenant) {
            return Ok(client.clone());
        }

        let client =
            Arc::new(Client::new(tenant.to_string(), polling_interval, hostname).await?);
        factory.insert(tenant.to_string(), client.clone());
        return Ok(client.clone());
    }

    pub fn get_client(&self, tenant: String) -> Result<Arc<Client>, String> {
        let factory = match self.read() {
            Ok(factory) => factory,
            Err(e) => {
                log::error!("CAC_CLIENT_FACTORY: failed to acquire read lock {}", e);
                return Err("CAC_CLIENT_FACTORY: Failed to acquire client.".to_string());
            }
        };

        match factory.get(&tenant) {
            Some(client) => Ok(client.clone()),
            None => Err("No such tenant found".to_string()),
        }
    }
}

use once_cell::sync::Lazy;
pub static CLIENT_FACTORY: Lazy<ClientFactory> =
    Lazy::new(|| ClientFactory(RwLock::new(HashMap::new())));

pub use eval::eval_cac;
pub use eval::eval_cac_with_reasoning;
pub use eval::merge;
