use std::time::Instant;

pub struct NamedInstant {
    name: String,
    instant: Instant,
}

impl NamedInstant {
    pub fn now(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            instant: Instant::now(),
        }
    }

    pub fn report_in_ms(&self) {
        println!(
            "'{}' took {} ms",
            self.name,
            self.instant.elapsed().as_millis()
        );
    }
}
