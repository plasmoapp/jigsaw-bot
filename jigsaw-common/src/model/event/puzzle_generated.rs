use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct PuzzleGeneratedEvent {
    pub request_uuid: Uuid,
    pub puzzle_uuid: Option<Uuid>,
}

impl PuzzleGeneratedEvent {
    pub fn new(request_uuid: Uuid, puzzle_uuid: Option<Uuid>) -> Self {
        Self {
            request_uuid,
            puzzle_uuid,
        }
    }
}
