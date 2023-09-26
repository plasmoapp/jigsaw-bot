use uuid::Uuid;

pub struct RedisScheme;

impl RedisScheme {
    pub fn jigsaw_puzzle_state(puzzle_uuid: &Uuid) -> String {
        format!("jigsaw_puzzle_state:{puzzle_uuid}")
    }
}
