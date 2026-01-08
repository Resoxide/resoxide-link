use serde::{Deserialize,Serialize};
use crate::data_model::{Component, Slot};

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotData {
    pub source_message_id: String,
    pub success: bool,
    pub error_info: Option<String>,
    pub depth: i32,
    pub data: Slot,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentData {
    pub source_message_id: String,
    pub success: bool,
    pub error_info: Option<String>,
    pub component: Component,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(tag = "$type")]
#[serde(rename_all = "camelCase")]
pub enum Response {
    Response {
        source_message_id: String,
        success: bool,
        error_info: Option<String>,
    },
    SlotData(SlotData),
    ComponentData(ComponentData),
}
