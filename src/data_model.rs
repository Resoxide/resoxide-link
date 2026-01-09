use std::collections::HashMap;
#[allow(unused_imports)]
use rust_decimal::Decimal;
pub use resoxide_json::{Json, Token, Error as JsonError};

#[derive(Default,Debug,Json)]
pub struct Reference {
    pub id: Option<String>,
    pub target_id: Option<String>,
    pub target_type: Option<String>,
}

impl From<Reference> for Member {
    fn from(value: Reference) -> Self {
        Self::Reference(value)
    }
}

#[derive(Debug,Json)]
pub struct SyncList {
    pub id: Option<String>,
    pub elements: Vec<Member>,
}

impl From<SyncList> for Member {
    fn from(value: SyncList) -> Self {
        Self::List(value)
    }
}

#[derive(Debug,Json)]
pub struct SyncObject {
    pub id: Option<String>,
    pub members: HashMap<String, Member>,
}

impl From<SyncObject> for Member {
    fn from(value: SyncObject) -> Self {
        Self::SyncObject(value)
    }
}

#[derive(Debug,Json)]
pub struct FieldEnum {
    pub id: Option<String>,
    pub value: String,
    pub enum_type: String,
}

impl From<FieldEnum> for Member {
    fn from(value: FieldEnum) -> Self {
        Self::Enum(value)
    }
}

#[derive(Json,Clone,Copy,Default,Debug,PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Json,Clone,Default,Debug,PartialEq)]
pub struct ColorX {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub profile: String,
}

#[derive(Json,Clone,Copy,Default,Debug,PartialEq,Eq,Hash)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone,Copy,Default,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
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

impl Json for Char {
    type Error = JsonError;

    fn to_token(&self) -> resoxide_json::Result<Token, Self::Error> {
        Ok(Token::String(self.to_string()))
    }

    fn from_token(token: &Token) -> resoxide_json::Result<Self, Self::Error> {
        match token {
            Token::String(s) if s.chars().count() == 1 => Ok(Char::try_new(s.chars().next().unwrap()).ok_or(JsonError)?),
            _ => Err(JsonError),
        }
    }

    fn error() -> Self::Error {
        JsonError
    }
}

include!(concat!(env!("OUT_DIR"), "/types.rs"));

impl From<&str> for Member {
    fn from(value: &str) -> Self {
        Self::String(FieldString { id: None, value: Some(value.to_string()) })
    }
}

#[derive(Default,Debug,Json)]
pub struct Component {
    pub id: Option<String>,
    pub is_reference_only: bool,
    pub component_type: String,
    pub members: Option<HashMap<String, Member>>,
}

impl Component {
    pub fn new(component_type: String) -> Self {
        Self {
            id: None,
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

#[derive(Json,Debug)]
pub struct Slot {
    pub id: Option<String>,
    pub is_reference_only: bool,
    pub parent: Reference,
    pub position: FieldFloat3,
    pub rotation: FieldFloatQ,
    pub scale: FieldFloat3,
    pub is_active: FieldBool,
    pub is_persistent: FieldBool,
    pub name: FieldString,
    pub tag: FieldString,
    pub components: Option<Vec<Component>>,
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
                id: None,
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
                target_id: self.id.clone(),
                target_type: Some(Slot::TYPE_NAME.to_string()),
            },
            name: FieldString {
                id: None,
                value: Some(name),
            },
            ..Default::default()
        }
    }
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            id: None,
            is_reference_only: false,
            parent: Reference {
                id: None,
                target_id: None,
                target_type: None,
            },
            position: FieldFloat3 {
                id: None,
                value: Default::default()
            },
            rotation: FieldFloatQ {
                id: None,
                value: Default::default()
            },
            scale: FieldFloat3 {
                id: None,
                value: Float3 { x: 1.0, y: 1.0, z: 1.0 },
            },
            is_active: FieldBool {
                id: None,
                value: true,
            },
            is_persistent: FieldBool {
                id: None,
                value: true,
            },
            name: FieldString {
                id: None,
                value: None,
            },
            tag: FieldString {
                id: None,
                value: None,
            },
            components: None,
            children: None,
        }
    }
}
