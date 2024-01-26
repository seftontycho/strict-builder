#![allow(dead_code)]

use strict_builder::Builder;

#[derive(Builder)]
pub struct Player {
    name: String,
    age: u8,
    #[builder(each = "friend")]
    friends: Vec<String>,
    #[builder(each = "siblings")]
    siblings: Vec<String>,
    swimming: bool,
    running: bool,
}

fn main() {
    let _ = Player::builder();
}
