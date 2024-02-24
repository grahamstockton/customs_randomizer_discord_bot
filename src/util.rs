use anyhow::{Error, Result};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use tabled::{builder::Builder as TableBuilder, settings::Width};

use crate::errors::InvalidInputError;

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

// Parses and validates input string. Allows for pseudonyms from pseudonyms.
pub fn parse_and_validate(input: &String, pseudonyms: &Map<String, Value>) -> Result<(HashSet<String>, HashSet<String>)> {
    let mut parts = input.split('/');
    let left_part = parts.next().expect("Unable to separate teams. See formatting rules with /help");
    let right_part = parts.next().expect("Unable to separate teams. See formatting rules with /help");
    let team_1: HashSet<String> = left_part.split(',').map(|e| replace_if_pseudonym(e.to_owned(), pseudonyms)).collect();
    let team_2: HashSet<String> = right_part.split(',').map(|e| replace_if_pseudonym(e.to_owned(), pseudonyms)).collect();

    // validate inputs
    if !(team_1.len() == 5 && team_2.len() == 5) {
        return Err(Error::from(InvalidInputError(String::from(
            "Failed to separate teams. Please see /help for formatting rules",
        ))));
    }

    Ok((team_1, team_2))
}

fn replace_if_pseudonym(ps: String, pseudonyms: &Map<String, Value>) -> String {
    if let Some(res) = pseudonyms.get(&ps) {
        return res.as_str().unwrap().to_owned();
    }

    ps
}