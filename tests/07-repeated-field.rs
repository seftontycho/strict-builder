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
    let mut builder = Player::builder();

    builder
        .name("Kvothe".to_string())
        .age(16)
        .friends(vec!["Willem".to_string(), "Simmon".to_string()])
        .friend("Devi".to_string())
        .siblings("Auri".to_string())
        .siblings("Devi".to_string())
        .swimming(true)
        .running(false);

    let player = builder.build().unwrap();

    assert_eq!(player.name, "Kvothe");
    assert_eq!(player.age, 16);
    assert_eq!(
        player.friends,
        vec![
            "Willem".to_string(),
            "Simmon".to_string(),
            "Devi".to_string()
        ]
    );
    assert_eq!(
        player.siblings,
        vec!["Auri".to_string(), "Devi".to_string()]
    );
    assert_eq!(player.swimming, true);
    assert_eq!(player.running, false);
}
