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
    let mut builder = Player::builder();

    builder.name("Kvothe".to_string());
    builder.age(16);
    builder.friends(vec!["Willem".to_string(), "Simmon".to_string()]);
    builder.swimming(true);
    builder.running(false);
}
