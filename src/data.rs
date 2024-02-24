use crate::riot_api::RiotApiAccessor;
use multimap::MultiMap;
use riven::RiotApi;
use serde_json::{Map, Value};
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
    pub pseudonyms: Map<String, Value>,
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
    pub fn new(riot_client: RiotApi, jg_champs: HashSet<String>, pseudonyms: Map<String, Value>) -> Self {
        Self {
            mutex: Mutex::new(MutexData::new()),
            riot_client: RiotApiAccessor::new(riot_client),
            jg_champs,
            pseudonyms
        }
    }

    pub fn clone_mutex_data(&self) -> MutexData {
        self.mutex.lock().unwrap().clone()
    }

    pub fn write_mutex_data(&self, data: MutexData) {
        *self.mutex.lock().unwrap() = data;
    }
}
