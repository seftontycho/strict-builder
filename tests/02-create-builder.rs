use strict_builder::Builder;

#[derive(Builder)]
pub struct Player {
    name: String,
    age: u8,
    friends: Vec<String>,
    swimming: bool,
    running: bool,
}

fn main() {
    let builder = Player::builder();

    let _ = builder;
}
