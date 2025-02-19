use derive_setters::Setters;
use monostate::{MustBe, MustBeU64};
use partial_id::Partial;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::resource::{resource, Endpoint};

use super::request::HttpRequest;
use super::{channel::Channel, resource::Snowflake, user::PartialUser};

#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq)]
pub struct MessageIdentifier {
    channel_id: Snowflake<Channel>,

    #[serde(rename = "id")]
    message_id: Snowflake<Message>,
}

impl MessageIdentifier {
    pub fn snowflake(&self) -> Snowflake<Message> {
        self.message_id
    }
}

#[derive(Partial)]
#[derive(Debug, Deserialize)]
pub struct Message {
    #[serde(flatten)]
    pub id: MessageIdentifier,

    pub author: PartialUser,
    pub content: String,

    #[serde(default)]
    pub embeds: Vec<Embed>,
    #[serde(default)]
    pub components: Vec<ActionRow>,
}

#[derive(Default, Setters, Serialize)]
#[setters(strip_option)]
pub struct CreateMessage {
    content: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    embeds: Vec<Embed>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    components: Vec<ActionRow>,
}

#[derive(Default, Setters, Serialize)]
#[setters(strip_option)]
pub struct PatchMessage {
    content: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    embeds: Vec<Embed>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    components: Vec<ActionRow>,
}

#[derive(Debug, Default, Setters, Serialize, Deserialize)]
#[setters(strip_option)]
pub struct Embed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub color: Option<u32>,
    pub author: Option<Author>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fields: Vec<Field>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionRow {
    #[serde(rename = "type")]
    typ: MustBe!(1u64),
    pub components: Vec<ActionRowComponent>,
}

impl ActionRow {
    pub fn new(components: Vec<ActionRowComponent>) -> Self {
        Self {
            typ: MustBeU64,
            components,
        }
    }
    pub fn is_full(&self) -> bool {
        if self.components.len() >= 5 {
            return false;
        }
        return match self.components.first() {
            Some(ActionRowComponent::Button(_)) => false,
            None => false,
            _ => true,
        };
    }
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ButtonStyle {
    Primary = 1,
    Secondary = 2,
    Success = 3,
    Danger = 4,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Button {
    Action {
        style: ButtonStyle,
        custom_id: String,
        label: Option<String>,
        #[serde(skip_serializing_if = "std::ops::Not::not", default)]
        disabled: bool,
    },
    Link {
        style: MustBe!(5u64),
        url: String,
        label: Option<String>,
        #[serde(skip_serializing_if = "std::ops::Not::not", default)]
        disabled: bool,
    },
}

const fn _default_1() -> usize {
    1
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextSelectMenu {
    pub custom_id: String,
    pub options: Vec<SelectOption>,
    pub placeholder: Option<String>,
    #[serde(default = "_default_1")]
    pub min_values: usize,
    #[serde(default = "_default_1")]
    pub max_values: usize,
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub disabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActionRowComponent {
    #[serde(rename = 2)]
    Button(Button),
    #[serde(rename = 3)]
    TextSelectMenu(TextSelectMenu),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
}

impl Author {
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self { name: name.into() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub value: String,

    #[serde(skip_serializing_if = "std::ops::Not::not", default)]
    pub inline: bool,
}

impl Field {
    pub fn new<S1, S2>(name: S1, value: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            name: name.into(),
            value: value.into(),
            inline: false,
        }
    }
    pub fn inlined<S1, S2>(name: S1, value: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        Self {
            name: name.into(),
            value: value.into(),
            inline: true,
        }
    }
}

#[derive(Serialize)]
struct CreateThread {
    name: String,
}

impl Endpoint for MessageIdentifier {
    fn uri(&self) -> String {
        format!(
            "/channels/{}/messages/{}",
            self.channel_id.as_int(),
            self.message_id.as_int()
        )
    }
}

pub trait MessageResource: Sized {
    fn endpoint(&self) -> MessageIdentifier;

    #[resource(Message)]
    fn get(&self) -> HttpRequest<Message> {
        HttpRequest::get(self.endpoint().uri())
    }
    #[resource(Message)]
    fn patch(&self, data: PatchMessage) -> HttpRequest<Message> {
        HttpRequest::patch(self.endpoint().uri(), &data)
    }
    #[resource(())]
    fn delete(self) -> HttpRequest<()> {
        HttpRequest::delete(self.endpoint().uri())
    }

    #[resource(Channel)]
    fn start_thread(&self, name: String) -> HttpRequest<Channel> {
        HttpRequest::post(
            format!("{}/threads", self.endpoint().uri()),
            &CreateThread { name },
        )
    }
}

impl MessageResource for MessageIdentifier {
    fn endpoint(&self) -> MessageIdentifier {
        self.clone()
    }
}
impl MessageResource for Message {
    fn endpoint(&self) -> MessageIdentifier {
        self.id
    }
}
impl MessageResource for PartialMessage {
    fn endpoint(&self) -> MessageIdentifier {
        self.id
    }
}
