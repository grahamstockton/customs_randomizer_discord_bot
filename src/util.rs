use std::collections::{HashMap, HashSet};
use tabled::{builder::Builder as TableBuilder, settings::Width};

// generate a string for a text response to user
pub fn get_prettified_result(
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
