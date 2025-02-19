use std::fmt::{Display, Formatter};

use derive_setters::Setters;
use partial_id::Partial;
use serde::{Deserialize, Serialize};

use crate::guild::PartialGuild;
use crate::resource::{resource, Endpoint};

use super::{channel::Channel, request::HttpRequest, resource::Snowflake};

#[derive(Partial)]
#[derive(Debug, Deserialize)]
pub struct User {
    pub id: Snowflake<User>,
    pub username: String,
}

impl Display for Snowflake<User> {
    fn fmt(&self, f: &mut Formatter<'_>) -> ::std::fmt::Result {
        f.write_fmt(format_args!("<@{}>", self.as_int()))
    }
}

impl Endpoint for Snowflake<User> {
    fn uri(&self) -> String {
        format!("/users/{}", self.as_int())
    }
}

#[derive(Default, Setters, Serialize)]
#[setters(strip_option)]
pub struct PatchUser {
    username: Option<String>,
}

#[derive(Serialize)]
struct DMRequest {
    recipient_id: Snowflake<User>,
}

pub trait UserResource {
    fn endpoint(&self) -> Snowflake<User>;

    #[resource(User)]
    fn get(&self) -> HttpRequest<User> {
        HttpRequest::get(self.endpoint().uri())
    }

    #[resource(Channel)]
    fn create_dm(&self) -> HttpRequest<Channel> {
        HttpRequest::post(
            "/users/@me/channels",
            &DMRequest {
                recipient_id: self.endpoint().clone(),
            },
        )
    }
}

impl UserResource for Snowflake<User> {
    fn endpoint(&self) -> Snowflake<User> {
        self.clone()
    }
}
impl UserResource for User {
    fn endpoint(&self) -> Snowflake<User> {
        self.id
    }
}
impl UserResource for PartialUser {
    fn endpoint(&self) -> Snowflake<User> {
        self.id
    }
}

pub struct Me;

impl Me {
    #[resource(User)]
    pub fn get(&self) -> HttpRequest<User> {
        HttpRequest::get("/users/@me")
    }
    #[resource(User)]
    pub fn patch(&self, data: PatchUser) -> HttpRequest<User> {
        HttpRequest::patch("/users/@me", &data)
    }

    #[resource(Vec<PartialGuild>)]
    pub fn get_guilds(&self) -> HttpRequest<Vec<PartialGuild>> {
        HttpRequest::get("/users/@me/guilds")
    }
}
