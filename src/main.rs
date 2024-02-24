mod choose_champions;
mod commands;
mod data;
mod errors;
mod riot_api;
mod summoner_data;
mod util;

use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;

use anyhow::Context as _;
use anyhow::Error;
use data::Data;
use poise::serenity_prelude as serenity;
use riven::RiotApi;
use serde_json::Value;
use shuttle_secrets::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use std::fs;
use std::io::BufReader;
use std::path::Path;

type Context<'a> = poise::Context<'a, Data, Error>;

/**
* Runtime for our program. Runs on poise, a runtime framework for creating discord bots. Deploys
* using shuttle.
*/
#[shuttle_runtime::main]
async fn poise(#[shuttle_secrets::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    // get discord token and api key from secret store
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;
    let riot_api_key = secret_store
        .get("RIOT_API_TOKEN")
        .context("'RIOT_API_KEY' was not found")?;

    // get list of jungle champs from config file
    let jg_champs: HashSet<String>;
    {
        let file = File::open(Path::new("JgChamps.txt")).unwrap();
        let reader = BufReader::new(&file);
        jg_champs = reader.lines().collect::<Result<_, _>>().unwrap();
    }

    // get list of player pseudonyms from config file
    let config = fs::read_to_string("pseudonym.json")?;
    let parsed: Value = serde_json::from_str(&config).unwrap();
    let pseudonyms = parsed.as_object().unwrap().clone();

    // initialize riot api
    let riot_client = RiotApi::new(riot_api_key);

    // initialize poise framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::help(), commands::new_game(), commands::reroll()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                // register commands
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                // register slash commands
                let create_commands =
                    poise::builtins::create_application_commands(&framework.options().commands);
                serenity::Command::set_global_commands(ctx, create_commands).await?;

                Ok(Data::new(riot_client, jg_champs, pseudonyms))
            })
        })
        .build();

    // build client
    let client =
        serenity::ClientBuilder::new(discord_token, serenity::GatewayIntents::non_privileged())
            .framework(framework)
            .await
            .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}
