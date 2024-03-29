use multimap::MultiMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::{HashMap, HashSet};

// Object to wrap the copy of the champ map used for selection without replacement
// Previously, this was done via passing a mutable reference to functions. Now, it can be
// done by instantiating this class and then calling its methods.
pub struct ChooseAndRemoveManager {
    champ_map: MultiMap<String, String>,
    team_1: HashSet<String>,
    team_2: HashSet<String>,
    jg_champs: HashSet<String>,
}

impl ChooseAndRemoveManager {
    pub fn new(
        champ_map: MultiMap<String, String>,
        team_1: HashSet<String>,
        team_2: HashSet<String>,
        jg_champs: HashSet<String>,
    ) -> Self {
        Self {
            champ_map,
            team_1,
            team_2,
            jg_champs,
        }
    }

    pub fn select_champs_with_removal(&mut self) -> HashMap<String, String> {
        // choose and remove from mutable copy of map
        let mut result: HashMap<String, String> = HashMap::new();
        let team_1_clone = self.team_1.clone();
        let team_2_clone = self.team_2.clone();
        let players = team_1_clone.union(&team_2_clone);
        for player in players {
            result.insert(player.to_owned(), self.choose_and_remove(&player));
        }

        // ensure team has a jg champ
        self.apply_jg_requirement(&self.team_1.clone(), &mut result);
        self.apply_jg_requirement(&self.team_2.clone(), &mut result);

        result
    }

    // choose randomly from a vector and then eliminate the value for all keys
    // I considered things like a surjective mapping, but those are slower because hashes don't work
    fn choose_and_remove(&mut self, player: &String) -> String {
        let choice = self
            .champ_map
            .get_vec(player)
            .expect("player not found")
            .choose(&mut thread_rng())
            .expect("choosing random element from vector failed")
            .to_owned();
        self.champ_map.retain(|_, v| v != &choice);

        choice.to_owned()
    }

    // Helper function for select_champs_with_removal. MUTATES RESULT FIELD
    // verify that team has a jg. If not, add one by randomly replacing a team member's champ
    fn apply_jg_requirement(
        &mut self,
        team: &HashSet<String>,
        result: &mut HashMap<String, String>, // MUTATES THIS FIELD
    ) {
        if !self.team_has_jg(team, result) {
            // notify via stdout
            println!("replacing for jg champ");
            println!("Initial result: {:?}", result);

            let mut shuffled_team: Vec<&String> = team.iter().collect();
            shuffled_team.shuffle(&mut thread_rng());

            for player in shuffled_team {
                let player_jg_champs: Vec<String> = self
                    .champ_map
                    .get_vec(player)
                    .expect("player not found")
                    .iter()
                    .map(|s| s.to_owned())
                    .filter(|c| self.jg_champs.contains(c))
                    .collect();

                let new_jg_champ = player_jg_champs
                    .choose(&mut thread_rng())
                    .expect("error unwrapping random choice")
                    .to_owned();

                if !player_jg_champs.is_empty() {
                    result.insert(player.to_owned(), new_jg_champ);
                    println!("New result: {:?}", result);

                    // return early with updated jg
                    return;
                }
            }
        }
    }

    fn team_has_jg(&self, team: &HashSet<String>, result: &mut HashMap<String, String>) -> bool {
        team.iter()
            .any(|p| self.jg_champs.contains(result.get(p).unwrap()))
    }
}
