use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ConsulOption {
    pub addr: String,
    pub timeout_sec: u64,
    pub protocol: String,
}

//TODO 默认直接使用本地8500的端口，真实项目中需要修改为环境变量或者配置文件配置
impl Default for ConsulOption {
    fn default() -> Self {
        Self {
            addr: String::from("127.0.0.1:8500"),
            timeout_sec: 1u64,
            protocol: "http".to_string(),
        }
    }
}

/**
 * deregister_critical_service_after 服务critical多久之后将会被注销
 * http 检查url
 * interval 检查间隔
 */
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct HealthCheck {
    pub deregisterCriticalServiceAfter: String,
    pub http: String,
    pub interval: String,
}

impl HealthCheck {
    /**
     *  新建一个健康检查的参数，默认30m分钟废弃，20s一检查
     */
    pub fn new(http: String) -> Self {
        return Self {
            deregisterCriticalServiceAfter: "30m".to_string(),
            http: http,
            interval: "20s".to_string(),
        };
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Registration {
    pub name: String,
    pub id: String,
    pub tags: Vec<String>,
    pub address: String,
    pub port: i32,
    pub check: HealthCheck,
}

impl Registration {
    pub fn new(
        name: &str,
        id: &str,
        tags: Vec<&str>,
        addr: &str,
        port: i32,
        health_check: HealthCheck,
    ) -> Self {
        Self {
            name: name.to_string(),
            id: id.to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            address: addr.to_string(),
            port: port,
            check: health_check,
        }
    }
    // pub fn simple_with_tags(name: &str, tags: Vec<&str>, addr: &str, port: i32) -> Self {
    //     Self::new(name, name, tags, addr, port, None)
    // }

    // pub fn simple(name: &str, addr: &str, port: i32) -> Self {
    //     Self::simple_with_tags(name, vec![], addr, port)
    // }

    pub fn simple_with_health_check(
        name: &str,
        addr: &str,
        port: i32,
        health_check: HealthCheck,
    ) -> Self {
        Self::new(name, name, vec![], addr, port, health_check)
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
