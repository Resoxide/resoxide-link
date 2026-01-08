use crate::data_model::{FieldBool, FieldFloat3, FieldFloatQ, FieldString, Float3, Member, MemberRef, Reference, Slot};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

fn serialize_as_opt_member<'a,S,T>(value: &'a Option<T>, s: S) -> Result<S::Ok, S::Error>
where MemberRef<'a>: From<&'a T>, S: Serializer
{
    if let Some(v) = value {
        s.serialize_some(&MemberRef::from(v))
    } else {
        s.serialize_none()
    }
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSlot {
    pub message_id: String,
    pub slot_id: String,
    pub depth: i32,
    pub include_component_data: bool,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSlotData {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub parent: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub position: Option<FieldFloat3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub rotation: Option<FieldFloatQ>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub scale: Option<FieldFloat3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub is_active: Option<FieldBool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub is_persistent: Option<FieldBool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub name: Option<FieldString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
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

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSlot {
    pub message_id: String,
    pub data: AddSlotData,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSlotData {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub parent: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub position: Option<FieldFloat3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub rotation: Option<FieldFloatQ>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub scale: Option<FieldFloat3>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub is_active: Option<FieldBool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub is_persistent: Option<FieldBool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub name: Option<FieldString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "serialize_as_opt_member")]
    pub tag: Option<FieldString>,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSlot {
    pub message_id: String,
    pub data: Slot,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSlot {
    pub message_id: String,
    pub slot_id: String,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetComponent {
    pub message_id: String,
    pub component_id: String,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddComponentData {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub component_type: String,
    pub members: HashMap<String,Member>,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddComponent {
    pub message_id: String,
    pub container_slot_id: String,
    pub data: AddComponentData,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponentData {
    pub id: String,
    pub members: HashMap<String,Member>,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponent {
    pub message_id: String,
    pub data: UpdateComponentData,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveComponent {
    pub message_id: String,
    pub component_id: String,
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(tag = "$type")]
#[serde(rename_all = "camelCase")]
pub enum Message {
    GetSlot(GetSlot),
    AddSlot(AddSlot),
    UpdateSlot(UpdateSlot),
    RemoveSlot(RemoveSlot),
    GetComponent(GetComponent),
    AddComponent(AddComponent),
    UpdateComponent(UpdateComponent),
    RemoveComponent(RemoveComponent),
}
