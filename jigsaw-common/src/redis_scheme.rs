use uuid::Uuid;

pub struct RedisScheme;

impl RedisScheme {
    // Keys

    pub fn jigsaw_puzzle_state(puzzle_uuid: &Uuid) -> String {
        format!("jigsaw_puzzle_state:{puzzle_uuid}")
    }

    pub fn jigsaw_puzzle_meta(puzzle_uuid: &Uuid) -> String {
        format!("jigsaw_puzzle_meta:{puzzle_uuid}")
    }

    pub fn request_message_data(request_uuid: &Uuid) -> String {
        format!("request_message_data:{request_uuid}")
    }

    pub fn jigsaw_puzzle_score(puzzle_uuid: &Uuid) -> String {
        format!("jigsaw_puzzle_score:{puzzle_uuid}")
    }

    pub const JIGSAW_USER_DATA: &'static str = "jigsaw_user_data";

    // pub fn jigsaw_user_data(user_id: u64) -> String {
    //     format!("jigsaw_user_data:{user_id}")
    // }

    // PubSub Request

    pub const REQUEST_GENERATE_PUZZLE: &'static str = "request:generate_puzzle";

    // PubSub Event

    pub const EVENT_PUZZLE_GENERATED: &'static str = "event:puzzle_generated";
}
