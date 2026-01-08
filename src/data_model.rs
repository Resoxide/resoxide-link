use std::collections::HashMap;
use std::fmt::Formatter;
use std::marker::PhantomData;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
#[allow(unused_imports)]
use rust_decimal::Decimal;
use serde::de::Error;

#[derive(Default,Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub target_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,
}

impl From<Reference> for Member {
    fn from(value: Reference) -> Self {
        Self::Reference(value)
    }
}

impl<'a> From<&'a Reference> for MemberRef<'a> {
    fn from(value: &'a Reference) -> Self {
        Self::Reference(value)
    }
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncList {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub elements: Vec<Member>,
}

impl From<SyncList> for Member {
    fn from(value: SyncList) -> Self {
        Self::List(value)
    }
}

impl<'a> From<&'a SyncList> for MemberRef<'a> {
    fn from(value: &'a SyncList) -> Self {
        Self::List(value)
    }
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncObject {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub members: HashMap<String, Member>,
}

impl From<SyncObject> for Member {
    fn from(value: SyncObject) -> Self {
        Self::SyncObject(value)
    }
}

impl<'a> From<&'a SyncObject> for MemberRef<'a> {
    fn from(value: &'a SyncObject) -> Self {
        Self::SyncObject(value)
    }
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldEnum {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub value: String,
    pub enum_type: String,
}

impl From<FieldEnum> for Member {
    fn from(value: FieldEnum) -> Self {
        Self::Enum(value)
    }
}

impl<'a> From<&'a FieldEnum> for MemberRef<'a> {
    fn from(value: &'a FieldEnum) -> Self {
        Self::Enum(value)
    }
}

#[derive(Clone,Copy,Default,Debug,Serialize,Deserialize,PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Clone,Default,Debug,Serialize,Deserialize,PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ColorX {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub profile: String,
}

#[derive(Clone,Copy,Default,Debug,Serialize,Deserialize,PartialEq,Eq,Hash)]
#[serde(rename_all = "camelCase")]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct Char(u16);

impl Char {
    pub fn new(value: u16) -> Char {
        if value >= 0xD800 && value < 0xE000 {
            panic!("Invalid codepoint for UTF-16");
        }
        Char(value)
    }

    pub fn new_lossy(value: u16) -> Char {
        if value >= 0xD800 && value < 0xE000 {
            Char(0xFFFD);
        }
        Char(value)
    }

    pub fn try_new(value: char) -> Option<Char> {
        if value >= '\u{10000}' {
            return None;
        }
        Some(Char(value as u16)) // char cannot be a surrogate
    }

    pub fn to_string(&self) -> String {
        String::from_utf16(&[self.0]).expect("Invariant only allows valid UTF-16")
    }
}

impl Serialize for Char {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Char {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        struct Visitor<'de>(PhantomData<&'de ()>);

        impl<'de> serde::de::Visitor<'de> for Visitor<'de> {
            type Value = Char;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "a string with a single character in the range D+0000-D+D7FFF or D+E000-D+FFFF")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error
            {
                let chars: Vec<_> = v.chars().collect();
                if chars.len() != 1 {
                    return Err(Error::custom(format!("Expected single character")));
                }
                if let Some(c) = Char::try_new(chars[0]) {
                    Ok(c)
                } else {
                    Err(Error::custom(format!("Character out of range")))
                }
            }
        }
        deserializer.deserialize_str(Visitor(PhantomData))
    }
}

include!(concat!(env!("OUT_DIR"), "/types.rs"));

impl From<&str> for Member {
    fn from(value: &str) -> Self {
        Self::String(FieldString { id: "".to_string(), value: Some(value.to_string()) })
    }
}

#[derive(Default,Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Component {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub is_reference_only: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub component_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<HashMap<String, Member>>,
}

impl Component {
    pub fn new(component_type: String) -> Self {
        Self {
            id: "".to_string(),
            is_reference_only: false,
            component_type,
            members: None,
        }
    }

    pub fn with_member(mut self, name: String, member: Member) -> Self {
        if matches!(self.members, None) {
            self.members = Some(HashMap::new());
        }
        self.members.as_mut().unwrap().insert(name, member);
        self
    }
}

pub fn serialize_as_member<'a,S,T>(value: &'a T, s: S) -> Result<S::Ok, S::Error>
where MemberRef<'a>: From<&'a T>, S: Serializer
{
    MemberRef::from(value).serialize(s)
}

#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Slot {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub is_reference_only: bool,
    #[serde(serialize_with = "serialize_as_member")]
    pub parent: Reference,
    #[serde(serialize_with = "serialize_as_member")]
    pub position: FieldFloat3,
    #[serde(serialize_with = "serialize_as_member")]
    pub rotation: FieldFloatQ,
    #[serde(serialize_with = "serialize_as_member")]
    pub scale: FieldFloat3,
    #[serde(serialize_with = "serialize_as_member")]
    pub is_active: FieldBool,
    #[serde(serialize_with = "serialize_as_member")]
    pub is_persistent: FieldBool,
    #[serde(serialize_with = "serialize_as_member")]
    pub name: FieldString,
    #[serde(serialize_with = "serialize_as_member")]
    pub tag: FieldString,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Slot>>,
}

impl Slot {
    pub const ROOT_SLOT_ID: &'static str = "Root";
    pub const TYPE_NAME: &'static str = "FrooxEngine.Slot";

    pub fn new(parent: &str, name: String) -> Self {
        Self {
            parent: Reference {
                id: None,
                target_id: Some(parent.to_string()),
                target_type: Some(Self::TYPE_NAME.to_string()),
            },
            name: FieldString {
                id: "".to_string(),
                value: Some(name),
            },
            ..Default::default()
        }
    }

    pub fn with_position(mut self, value: Float3) -> Self {
        self.position.value = value;
        self
    }

    pub fn with_rotation(mut self, value: FloatQ) -> Self {
        self.rotation.value = value;
        self
    }

    pub fn with_scale(mut self, value: Float3) -> Self {
        self.scale.value = value;
        self
    }

    pub fn with_tag(mut self, value: String) -> Self {
        self.tag.value = Some(value);
        self
    }

    pub fn add_component(&mut self, component: Component) {
        if matches!(self.components, None) {
            self.components = Some(vec![]);
        }
        self.components.as_mut().unwrap().push(component);
    }

    pub fn add_child(&mut self, name: String) -> Self {
        Self {
            parent: Reference {
                id: None,
                target_id: Some(self.id.to_string()),
                target_type: Some("Slot".to_string()),
            },
            name: FieldString {
                id: "".to_string(),
                value: Some(name),
            },
            ..Default::default()
        }
    }
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            is_reference_only: false,
            parent: Reference {
                id: None,
                target_id: None,
                target_type: None,
            },
            position: FieldFloat3 {
                id: "".to_string(),
                value: Default::default()
            },
            rotation: FieldFloatQ {
                id: "".to_string(),
                value: Default::default()
            },
            scale: FieldFloat3 {
                id: "".to_string(),
                value: Float3 { x: 1.0, y: 1.0, z: 1.0 },
            },
            is_active: FieldBool {
                id: "".to_string(),
                value: true,
            },
            is_persistent: FieldBool {
                id: "".to_string(),
                value: true,
            },
            name: FieldString {
                id: "".to_string(),
                value: None,
            },
            tag: FieldString {
                id: "".to_string(),
                value: None,
            },
            components: None,
            children: None,
        }
    }
}

