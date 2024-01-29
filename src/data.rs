use crate::riot_api::RiotApiAccessor;
use multimap::MultiMap;
use riven::RiotApi;
use std::{collections::HashSet, sync::Mutex};

// Context data object for poise. Gets passed to every command and holds state for our program
#[derive(Clone)]
pub struct MutexData {
    pub team_1: HashSet<String>,
    pub team_2: HashSet<String>,
    pub player_champ_map: MultiMap<String, String>,
}

pub struct Data {
    pub mutex: Mutex<MutexData>,
    pub riot_client: RiotApiAccessor,
    pub jg_champs: HashSet<String>,
}

impl MutexData {
    pub fn new() -> Self {
        Self {
            team_1: HashSet::<String>::new(),
            team_2: HashSet::<String>::new(),
            player_champ_map: MultiMap::new(),
        }
    }
}

impl Data {
    pub fn new(riot_client: RiotApi, jg_champs: HashSet<String>) -> Self {
        Self {
            mutex: Mutex::new(MutexData::new()),
            riot_client: RiotApiAccessor::new(riot_client),
            jg_champs,
        }
    }
}
