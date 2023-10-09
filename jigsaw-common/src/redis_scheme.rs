use uuid::Uuid;

pub struct RedisScheme;

impl RedisScheme {
    // Keys

    // HSET where field is tile_uuid and value is JigsawTile
    pub fn jigsaw_puzzle_state(puzzle_uuid: &Uuid) -> String {
        format!("jigsaw_puzzle_state:{puzzle_uuid}")
    }

    // Value is JigsawMeta
    pub fn jigsaw_puzzle_meta(puzzle_uuid: &Uuid) -> String {
        format!("jigsaw_puzzle_meta:{puzzle_uuid}")
    }

    //
    pub fn request_message_data(request_uuid: &Uuid) -> String {
        format!("request_message_data:{request_uuid}")
    }

    // ZSET where field is u64 UserId
    pub fn jigsaw_puzzle_score(puzzle_uuid: &Uuid) -> String {
        format!("jigsaw_puzzle_score:{puzzle_uuid}")
    }

    // Value is u64
    pub fn jigsaw_puzzle_score_total(puzzle_uuid: &Uuid) -> String {
        format!("jigsaw_puzzle_score_total:{puzzle_uuid}")
    }

    // HSET where field is u64 UserId and value is UserData
    pub const JIGSAW_USER_DATA: &'static str = "jigsaw_user_data";

    // PubSub Request

    pub const REQUEST_GENERATE_PUZZLE: &'static str = "request:generate_puzzle";

    // PubSub Event

    pub const EVENT_PUZZLE_GENERATED: &'static str = "event:puzzle_generated";
}
