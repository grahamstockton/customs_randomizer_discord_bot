use anyhow::Result;
use riven::models::summoner_v4::Summoner;
use std::collections::HashSet;

use crate::riot_api::RiotApiAccessor;

pub struct SummonerData {
    name: String,
    tagline: String,
    pub mastered_champs: HashSet<String>,
    pub summoner: Summoner,
}

impl SummonerData {
    pub async fn new(player: &str, riot_client: &RiotApiAccessor) -> Result<Self> {
        // split summoner name for lookup
        let mut parts = player.split('#');
        let name = parts
            .next()
            .expect("Failed to split summoner name")
            .to_owned();
        let tagline = parts
            .next()
            .expect("Failed to split summoner name")
            .to_owned();

        // get account data from riot api
        let account = riot_client.get_account(&name, &tagline).await?;

        // get summoner data from account data. Needed for summoner level
        let summoner = riot_client.get_summoner(&account.puuid).await?;

        // get mastered_champs for that puuid from riot api
        let mastered_champs: HashSet<String> = riot_client
            .get_mastered_champs(&account.puuid)
            .await?
            .iter()
            .map(|e| e.champion_id.name().unwrap_or("UNKNOWN").to_owned())
            .collect();

        Ok(SummonerData {
            name,
            tagline,
            mastered_champs,
            summoner,
        })
    }

    pub fn get_riot_id(&self) -> String {
        self.name.clone() + "#" + &self.tagline
    }
}
