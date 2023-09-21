use shrinkwraprs::Shrinkwrap;
use uuid::Uuid;

#[derive(Shrinkwrap)]
pub struct Indexed<I, T> {
    pub id: I,
    #[shrinkwrap(main_field)]
    pub value: T,
}

impl<T> From<T> for Indexed<Uuid, T> {
    fn from(value: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            value,
        }
    }
}
