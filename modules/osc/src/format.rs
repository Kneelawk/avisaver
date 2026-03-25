//! OSCQuery JSON formats.
//!
//! See https://github.com/Vidvox/OSCQueryProposal

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

/// Host-Info json format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSCQHostInfo {
    #[serde(rename = "NAME", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "EXTENSIONS", default)]
    pub extensions: HashMap<String, bool>,
    #[serde(rename = "OSC_IP", skip_serializing_if = "Option::is_none")]
    pub osc_ip: Option<String>,
    #[serde(rename = "OSC_PORT", skip_serializing_if = "Option::is_none")]
    pub osc_port: Option<u16>,
    #[serde(rename = "OSC_TRANSPORT", default = "default_osc_transport")]
    pub osc_transport: String,
}

impl Default for OSCQHostInfo {
    fn default() -> Self {
        Self {
            name: None,
            extensions: HashMap::from([("ACCESS".to_string(), true)]),
            osc_ip: Some("127.0.0.1".to_string()),
            osc_port: None,
            osc_transport: default_osc_transport(),
        }
    }
}

fn default_osc_transport() -> String {
    "UDP".to_string()
}

/// Query node json format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSCQNode {
    #[serde(rename = "FULL_PATH")]
    pub full_path: String,
    #[serde(rename = "CONTENTS", skip_serializing_if = "HashMap::is_empty")]
    pub contents: HashMap<String, OSCQNode>,
    #[serde(rename = "TYPE", skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
    #[serde(rename = "ACCESS", skip_serializing_if = "Option::is_none")]
    pub access: Option<u32>,
    #[serde(rename = "VALUE", skip_serializing_if = "Option::is_none")]
    pub value: Option<Vec<OSCValue>>,
}

impl Default for OSCQNode {
    fn default() -> Self {
        Self {
            full_path: "/".to_string(),
            contents: Default::default(),
            ty: None,
            access: None,
            value: None,
        }
    }
}

/// Query value json format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, PartialEq)]
#[serde(untagged)]
pub enum OSCValue {
    Number(f64),
    Boolean(bool),
    String(String),
}

impl Display for OSCValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OSCValue::Number(n) => write!(f, "{}", n),
            OSCValue::Boolean(b) => write!(f, "{}", b),
            OSCValue::String(s) => write!(f, "{}", s),
        }
    }
}
