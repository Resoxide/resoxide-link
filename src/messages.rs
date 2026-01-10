use crate::data_model::{FieldBool, FieldFloat3, FieldFloatQ, FieldString, Float3, Member, Reference, Slot};
use std::collections::HashMap;
use resoxide_json::Json;

#[derive(Debug,Default,Json)]
pub struct GetSlot {
    pub message_id: String,
    pub slot_id: String,
    pub depth: i32,
    pub include_component_data: bool,
}

#[derive(Debug,Default,Json)]
pub struct AddSlotData {
    pub id: Option<String>,
    pub parent: Option<Reference>,
    pub position: Option<FieldFloat3>,
    pub rotation: Option<FieldFloatQ>,
    pub scale: Option<FieldFloat3>,
    pub is_active: Option<FieldBool>,
    pub is_persistent: Option<FieldBool>,
    pub name: Option<FieldString>,
    pub tag: Option<FieldString>,
}

impl From<Slot> for AddSlotData {
    fn from(value: Slot) -> Self {
        Self {
            id: value.id,
            parent: Some(value.parent),
            position: if value.position.value == Default::default() { None } else { Some(value.position) },
            rotation: if value.rotation.value == Default::default() { None } else { Some(value.rotation) },
            scale: if value.scale.value == Float3::ONES { None } else { Some(value.scale) },
            is_active: if value.is_active.value { None } else { Some(value.is_active) },
            is_persistent: if value.is_persistent.value { None } else { Some(value.is_persistent) },
            name: if value.name.value.is_none() { None } else { Some(value.name) },
            tag: if value.tag.value.is_none() { None } else { Some(value.tag) },
        }
    }
}

#[derive(Debug,Default,Json)]
pub struct AddSlot {
    pub message_id: String,
    pub data: AddSlotData,
}

#[derive(Debug,Default,Json)]
pub struct UpdateSlotData {
    pub id: String,
    pub parent: Option<Reference>,
    pub position: Option<FieldFloat3>,
    pub rotation: Option<FieldFloatQ>,
    pub scale: Option<FieldFloat3>,
    pub is_active: Option<FieldBool>,
    pub is_persistent: Option<FieldBool>,
    pub name: Option<FieldString>,
    pub tag: Option<FieldString>,
}

#[derive(Debug,Default,Json)]
pub struct UpdateSlot {
    pub message_id: String,
    pub data: UpdateSlotData,
}

#[derive(Debug,Default,Json)]
pub struct RemoveSlot {
    pub message_id: String,
    pub slot_id: String,
}

#[derive(Debug,Default,Json)]
pub struct GetComponent {
    pub message_id: String,
    pub component_id: String,
}

#[derive(Debug,Default,Json)]
pub struct AddComponentData {
    pub id: Option<String>,
    pub component_type: String,
    pub members: HashMap<String,Member>,
}

#[derive(Debug,Default,Json)]
pub struct AddComponent {
    pub message_id: String,
    pub container_slot_id: String,
    pub data: AddComponentData,
}

#[derive(Debug,Default,Json)]
pub struct UpdateComponentData {
    pub id: String,
    pub members: HashMap<String,Member>,
}

#[derive(Debug,Default,Json)]
pub struct UpdateComponent {
    pub message_id: String,
    pub data: UpdateComponentData,
}

#[derive(Debug,Default,Json)]
pub struct RemoveComponent {
    pub message_id: String,
    pub component_id: String,
}

#[derive(Debug,Default,Json)]
pub struct ImportTexture2DFile {
    pub message_id: String,
    pub file_path: String,
}

#[derive(Debug,Default,Json)]
pub struct ImportTexture2DRawData {
    pub message_id: String,
    pub width: i32,
    pub height: i32,
    pub color_profile: String,
}

#[derive(Debug,Default,Json)]
pub struct ImportTexture2DRawDataHDR {
    pub message_id: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug,Json)]
pub enum Message {
    GetSlot(GetSlot),
    AddSlot(AddSlot),
    UpdateSlot(UpdateSlot),
    RemoveSlot(RemoveSlot),
    GetComponent(GetComponent),
    AddComponent(AddComponent),
    UpdateComponent(UpdateComponent),
    RemoveComponent(RemoveComponent),
    #[json(rename = "importTexture2DFile")]
    ImportTexture2DFile(ImportTexture2DFile),
    #[json(rename = "importTexture2DRawData")]
    ImportTexture2DRawData(ImportTexture2DRawData),
    #[json(rename = "importTexture2DRawDataHDR")]
    ImportTexture2DRawDataHDR(ImportTexture2DRawDataHDR),
}
