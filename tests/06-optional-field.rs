use strict_builder::Builder;

#[derive(Builder)]
pub struct Player {
    name: String,
    age: u8,
    friends: Vec<String>,
    siblings: Option<Vec<String>>,
    swimming: bool,
    running: bool,
}

fn main() {
    // test optional field is None by default
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
    assert_eq!(player.siblings, None);
    assert_eq!(player.swimming, true);
    assert_eq!(player.running, false);

    let mut builder = Player::builder();

    builder
        .name("Kvothe".to_string())
        .age(16)
        .friends(vec!["Willem".to_string(), "Simmon".to_string()])
        .siblings(vec!["Devi".to_string(), "Auri".to_string()])
        .swimming(true)
        .running(false);

    // test can set optional field
    let player = builder.build().unwrap();

    assert_eq!(player.name, "Kvothe");
    assert_eq!(player.age, 16);
    assert_eq!(
        player.friends,
        vec!["Willem".to_string(), "Simmon".to_string()]
    );
    assert_eq!(
        player.siblings,
        Some(vec!["Devi".to_string(), "Auri".to_string()])
    );
    assert_eq!(player.swimming, true);
    assert_eq!(player.running, false);
}
