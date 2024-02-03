use crate::data::MutexData;
use crate::errors::InvalidInputError;
use crate::riot_api::RiotApiAccessor;
use crate::summoner_data::SummonerData;
use crate::Context;
use anyhow::{Error, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use multimap::MultiMap;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};
use tabled::{builder::Builder as TableBuilder, settings::Width};

const HELP_MESSAGE: &str =
    "To randomize champs, type /new_game followed by lists of summoners for both teams.
The list should be of the form \"summ1#NA1,summ2#NA1,summ3#JP1,summ4#YAY,summ5#h/summ6#NA1,summ7#B,summ8#1,summ9#NA1,summ10#NA1\".
Because Riot is dumb, you need to use your full id (e.g. MyId#Tagline). For most, the tagline is #NA1.
If you have already entered the list of summoners during this session, you can use /reroll. Have fun! :3c";

#[poise::command(slash_command)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(HELP_MESSAGE).await?;

    Ok(())
}

/**
* Take a list of players, find what champs are available to them, then assign champions
*/
#[poise::command(slash_command)]
pub async fn new_game(
    ctx: Context<'_>,
    #[description = "A list of players. To see the formatting, use /help"] input: String,
) -> Result<(), Error> {
    ctx.say("Generating randomized champions").await?;

    // create players lists
    // TODO: move this into a function
    let mut parts = input.split('/');
    let left_part = parts.next().expect("Error splitting command string");
    let right_part = parts
        .next()
        .expect("Unable to separate teams. See formatting rules with /help");
    let team_1: HashSet<String> = left_part.trim().split(',').map(|e| e.to_owned()).collect();
    let team_2: HashSet<String> = right_part.trim().split(',').map(|e| e.to_owned()).collect();

    // validate inputs
    if !(team_1.len() == 5 && team_2.len() == 5) {
        return Err(Error::from(InvalidInputError(String::from(
            "Failed to separate teams. Please see /help for formatting rules",
        ))));
    }

    // get a copy of data inside the mutex
    let mutex_copy: MutexData;
    {
        let data = ctx.data().mutex.lock().unwrap();
        mutex_copy = data.clone();
    }

    // update data if teams have changed or this is the first roll
    let mut player_champ_map = mutex_copy.player_champ_map;
    if !(team_1 == mutex_copy.team_1 && team_2 == mutex_copy.team_2) {
        // generate new player champ map
        let players: HashSet<&String> = team_1.union(&team_2).collect();
        player_champ_map = get_player_champion_map(&players, &ctx.data().riot_client).await?;

        // update mutex
        {
            let mut data = ctx.data().mutex.lock().unwrap();
            *data = MutexData {
                team_1: team_1.clone(),
                team_2: team_2.clone(),
                player_champ_map: player_champ_map.clone(),
            };
        }
    }

    // tell the requester the champ selection
    ctx.say(get_prettified_result(
        &team_1,
        &team_2,
        &select_champs_and_remove(&team_1, &team_2, player_champ_map, &ctx.data().jg_champs),
    ))
    .await?;

    Ok(())
}

// If no need to update data, we can redo champ selection only
#[poise::command(slash_command)]
pub async fn reroll(ctx: Context<'_>) -> Result<(), Error> {
    // read from mutex
    let mutex_copy: MutexData;
    {
        let data = ctx.data().mutex.lock().unwrap();
        mutex_copy = data.clone();
    }

    // check if uninitialized
    // If this project becomes bigger, should probably create a better model and validation
    if mutex_copy.team_1.is_empty() {
        ctx.say("Teams are uninitialized. Please run /new_game. For more details, use /help")
            .await?;
        return Ok(());
    }

    // tell the requester the champ selection
    ctx.say(get_prettified_result(
        &mutex_copy.team_1,
        &mutex_copy.team_2,
        &select_champs_and_remove(
            &mutex_copy.team_1,
            &mutex_copy.team_2,
            mutex_copy.player_champ_map,
            &ctx.data().jg_champs,
        ),
    ))
    .await?;

    Ok(())
}

// generate a mapping from summoner name to champions they are confirmed to have access to
async fn get_player_champion_map(
    players: &HashSet<&String>,
    riot_client: &RiotApiAccessor,
) -> Result<MultiMap<String, String>> {
    let mut return_map = MultiMap::<String, String>::new();

    // get free to play champs list for both new (level < 10) and regular players
    let rotation_object = riot_client.get_rotation_champs().await?;

    // make calls concurrently to speed up initial deta collection
    let mut futures =
        FuturesUnordered::from_iter(players.iter().map(|p| SummonerData::new(p, riot_client)));
    while let Some(s) = futures.next().await {
        let summoner_data = s.unwrap();
        let riot_id = summoner_data.get_riot_id();

        // get player free to play champs list
        let rotation_champs: Vec<String> = if rotation_object.max_new_player_level as i64
            > summoner_data.summoner.summoner_level
        {
            rotation_object
                .free_champion_ids_for_new_players
                .iter()
                .map(|e| e.name().unwrap_or("UNKOWN").to_owned())
                .collect()
        } else {
            rotation_object
                .free_champion_ids
                .iter()
                .map(|e| e.name().unwrap_or("UNKNOWN").to_owned())
                .collect()
        };

        // create return map from mastered champs and free to play champs
        return_map.insert_many(riot_id.clone(), summoner_data.mastered_champs);
        return_map.insert_many(riot_id, rotation_champs);
    }

    Ok(return_map)
}

// method containing shared logic between new_game and reroll
fn select_champs_and_remove(
    team_1: &HashSet<String>,
    team_2: &HashSet<String>,
    mut champ_map: MultiMap<String, String>,
    jg_champs: &HashSet<String>,
) -> HashMap<String, String> {
    let mut rng = thread_rng();

    let ref_map = champ_map.clone();
    let mut result: HashMap<String, String> = ref_map
        .iter_all()
        .map(|(e, v)| (e.to_owned(), choose_and_remove(&mut champ_map, v, &mut rng)))
        .collect();

    // ensure team has a jg champ
    for team in [team_1, team_2].iter() {
        apply_jg_requirement(team, &mut result, &mut champ_map, jg_champs, &mut rng);
    }

    result
}

// verify that team has a jg. If not, add one by randomly replacing a team member's champ
fn apply_jg_requirement(
    team: &HashSet<String>,
    result: &mut HashMap<String, String>,
    champ_map: &mut MultiMap<String, String>,
    jg_champs: &HashSet<String>,
    rng: &mut ThreadRng,
) {
    if !result.iter().any(|(_, v)| jg_champs.contains(v)) {
        let mut shuffled_team: Vec<&String> = team.iter().collect();
        shuffled_team.shuffle(rng);

        for player in shuffled_team {
            let player_jg_champs: Vec<String> = champ_map
                .get_vec(player)
                .unwrap()
                .iter()
                .map(|s| s.to_owned())
                .filter(|c| jg_champs.contains(c))
                .collect();

            if !player_jg_champs.is_empty() {
                result.insert(
                    player.to_owned(),
                    player_jg_champs.choose(rng).unwrap().to_owned(),
                );

                // return early with updated jg
                return;
            }
        }
    }
}

// choose randomly from a vector and then eliminate the value for all keys
// I considered things like a surjective mapping, but those are slower because hashes don't work
fn choose_and_remove(
    champ_map: &mut MultiMap<String, String>,
    choose_vec: &Vec<String>,
    rng: &mut ThreadRng,
) -> String {
    let c = choose_vec.choose(rng).unwrap();
    let m = champ_map.clone();
    for k in m.keys() {
        let v = champ_map.get_vec_mut(k).unwrap();
        if let Some(found_idx) = v.iter().position(|r| r == c) {
            v.remove(found_idx);
        }
    }

    c.to_owned()
}

// generate a string for a text response to user
fn get_prettified_result(
    team_1: &HashSet<String>,
    team_2: &HashSet<String>,
    selection: &HashMap<String, String>,
) -> String {
    let mut builder = TableBuilder::default();
    for (left, right) in team_1.iter().zip(team_2) {
        builder.push_record(vec![left, &selection[left], right, &selection[right]]);
    }

    builder.build().with(Width::justify(25)).to_string()
}
