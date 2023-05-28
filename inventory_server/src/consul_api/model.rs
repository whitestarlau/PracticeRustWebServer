use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ConsulOption {
    pub addr: String,
    pub timeout_sec: u64,
    pub protocol: String,
}
impl Default for ConsulOption {
    fn default() -> Self {
        Self {
            addr: String::from("127.0.0.1:8500"),
            timeout_sec: 1u64,
            protocol: "http".to_string(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Registration {
    pub name: String,
    pub id: String,
    pub tags: Vec<String>,
    pub address: String,
    pub port: i32,
}

impl Registration {
    pub fn new(name: &str, id: &str, tags: Vec<&str>, addr: &str, port: i32) -> Self {
        Self {
            name: name.to_string(),
            id: id.to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            address: addr.to_string(),
            port,
        }
    }
    pub fn simple_with_tags(name: &str, tags: Vec<&str>, addr: &str, port: i32) -> Self {
        Self::new(name, name, tags, addr, port)
    }
    pub fn simple(name: &str, addr: &str, port: i32) -> Self {
        Self::simple_with_tags(name, vec![], addr, port)
    }
}
#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Service {
    #[serde(rename = "ID")]
    pub id: String,
    pub service: String,
    pub tags: Vec<String>,
    pub address: String,
    pub port: i32,
    pub datacenter: String,
}

pub type Services = HashMap<String, Service>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Filter {
    Service(String),
    ID(String),
}
