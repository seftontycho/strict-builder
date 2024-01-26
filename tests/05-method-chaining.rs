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

    builder
        .name("Kvothe".to_string())
        .age(16)
        .friends(vec!["Willem".to_string(), "Simmon".to_string()])
        .swimming(true)
        .running(false);

    let player = builder.build().unwrap();

    assert_eq!(player.name, "Kvothe");
    assert_eq!(player.age, 16);
    assert_eq!(
        player.friends,
        vec!["Willem".to_string(), "Simmon".to_string()]
    );
    assert_eq!(player.swimming, true);
    assert_eq!(player.running, false);
}
