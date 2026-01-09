use crate::data_model::{Component, Slot};
use resoxide_json::Json;

#[derive(Debug,Default,Json)]
pub struct SlotData {
    pub source_message_id: String,
    pub success: bool,
    pub error_info: Option<String>,
    pub depth: i32,
    pub data: Slot,
}

#[derive(Debug,Default,Json)]
pub struct ComponentData {
    pub source_message_id: String,
    pub success: bool,
    pub error_info: Option<String>,
    pub data: Component,
}

#[derive(Debug,Default,Json)]
pub struct ResponseData {
    source_message_id: String,
    success: bool,
    error_info: Option<String>,
}

#[derive(Debug,Json)]
pub enum Response {
    Response(ResponseData),
    SlotData(SlotData),
    ComponentData(ComponentData),
}
