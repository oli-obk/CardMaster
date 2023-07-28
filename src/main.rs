#![feature(try_trait_v2)]
#![feature(exhaustive_patterns)]

use async_trait::async_trait;
use cah::CAH;
use discord::command::{Param, StringOption};
use discord::interaction::{AnyInteraction, InteractionResource, MessageComponent, Webhook};
use discord::request::Discord;
use discord::user::User;
use dotenv_codegen::dotenv;
use futures_util::StreamExt;
use game::{Flow, Game, GameMessage, GameUI, InteractionDispatcher, Logic};

use discord::application::{self, ApplicationResource};
use discord::command::CommandData;
use discord::command::Commands;
use discord::gateway::Gateway;
use discord::gateway::GatewayEvent;
use discord::interaction::Interaction;
use discord::request::Result;
use discord::resource::{Creatable, Deletable, Resource};

mod cah;
mod game;

const RUSTMASTER: &str = dotenv!("RUSTMASTER");
const CARDMASTER: &str = dotenv!("CARDMASTER");

async fn purge(commands: Commands, client: &Discord) -> Result<()> {
    for command in commands.get(client).await? {
        command.delete(client).await?;
    }
    Ok(())
}

async fn on_command(i: AnyInteraction, d: &mut InteractionDispatcher) -> Result<()> {
    match i {
        AnyInteraction::Command(command) => match command.data.name.as_str() {
            "ping" => {
                command
                    .token
                    .reply(&Webhook, |m| m.content("hurb".to_owned()))
                    .await?;
            }
            "play" => {
                let game = command.data.options[0].as_string().unwrap();
                let task = match game {
                    TestGame::NAME => TestGame::start(command.token, command.user),
                    CAH::NAME => CAH::start(command.token, command.user),
                    _ => panic!("unknown game"),
                }
                .await?;
                d.register(task);
            }
            _ => {}
        },
        AnyInteraction::Component(comp) => d.dispatch(comp).await,
    };
    Ok(())
}

struct TestGame;

#[async_trait]
impl Logic for TestGame {
    type Return = ();
    async fn logic(&mut self, _ui: &mut GameUI, _i: Interaction<MessageComponent>) -> Flow<()> {
        Flow::Return(())
    }
}

#[async_trait]
impl Game for TestGame {
    const NAME: &'static str = "Test";
    const COLOR: u32 = 0xFFFFFF;

    fn new(user: User) -> Self {
        TestGame
    }

    fn lobby_msg_reply(&self) -> GameMessage {
        Self::message(Vec::new(), Vec::new())
    }
}

async fn run() -> Result<()> {
    // connect
    let client = Discord::new(RUSTMASTER);

    // create commands
    let application = application::Me.get(&client).await?;
    purge(application.global_commands(), &client).await?;

    application
        .global_commands()
        .create(&client, &CommandData::new("ping", "Replies with pong!"))
        .await?;

    application
        .global_commands()
        .create(
            &client,
            &CommandData::new("birthday", "Sets user birthday").options(vec![StringOption::new(
                "birthday",
                "Your birthday",
            )
            .required()
            .into()]),
        )
        .await?;

    application
        .global_commands()
        .create(
            &client,
            &CommandData::new("play", "Start a new game").options(vec![StringOption::new(
                "game",
                "What game to play",
            )
            .required()
            .choices(vec![
                Param::new(TestGame::NAME, TestGame::NAME),
                Param::new(CAH::NAME, CAH::NAME),
            ])
            .into()]),
        )
        .await?;

    // create dispatch
    let mut dispatch = InteractionDispatcher::new();

    // gateway
    let mut gateway = Gateway::connect(&client).await?;
    while let Some(event) = gateway.next().await {
        match event {
            GatewayEvent::InteractionCreate(i) => on_command(i, &mut dispatch).await?,
            _ => {}
        }
    }
    gateway.close().await;
    Ok(())
}

#[tokio::main]
async fn main() {
    run().await.unwrap()
}
