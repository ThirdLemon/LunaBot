use std::{collections::HashMap, fs::File, io::Write, sync::Arc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct BSTBlock {
    hp: u8,
    atk: u8,
    def: u8,
    spatk: u8,
    spdef: u8,
    spd: u8
}

#[derive(Serialize, Deserialize, Debug)]
struct SpeciesData {
    dex_number: u16,
    name: String,
    bst: BSTBlock,
    base_friendship: u8,
}