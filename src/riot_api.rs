use anyhow::Result;
use riven::consts::{PlatformRoute, RegionalRoute};
use riven::models::{
    account_v1::Account, champion_mastery_v4::ChampionMastery, champion_v3::ChampionInfo,
    summoner_v4::Summoner,
};
use riven::RiotApi;

pub struct RiotApiAccessor {
    riot_api: RiotApi,
}

/**
* Best effort attempt to get champions playable by user. Won't return owned champs that
* have never been played and aren't free-to-play, but will return everything else {
*/
impl RiotApiAccessor {
    // injection for actual api client
    pub fn new(riot_api: RiotApi) -> Self {
        RiotApiAccessor { riot_api }
    }

    // get player uuid given full riot id with tagline
    pub async fn get_account(&self, player_name: &str, tagline: &str) -> Result<Account> {
        let acc = self
            .riot_api
            .account_v1()
            .get_by_riot_id(RegionalRoute::AMERICAS, player_name, tagline)
            .await?
            .expect("failed to get account from player_name and tagline");

        Ok(acc)
    }

    // get list of played-before champions from puuid
    // TODO: Remove this once you can verify we don't actually need it. Can use Account.puuid
    pub async fn get_summoner(&self, puuid: &str) -> Result<Summoner> {
        let summ = self
            .riot_api
            .summoner_v4()
            .get_by_puuid(PlatformRoute::NA1, puuid)
            .await?;

        Ok(summ)
    }

    // get champion mastery list object from puuid
    pub async fn get_mastered_champs(&self, puuid: &str) -> Result<Vec<ChampionMastery>> {
        let ms = self
            .riot_api
            .champion_mastery_v4()
            .get_all_champion_masteries_by_puuid(PlatformRoute::NA1, puuid)
            .await?;

        Ok(ms)
    }

    // get champs currently offered for free for all players by riot
    pub async fn get_rotation_champs(&self) -> Result<ChampionInfo> {
        let rot = self
            .riot_api
            .champion_v3()
            .get_champion_info(PlatformRoute::NA1)
            .await?;

        Ok(rot)
    }
}
