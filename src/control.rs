use crate::types::Power;

pub struct LoadFollowing();

impl LoadFollowing {
    pub fn new() -> LoadFollowing {
        LoadFollowing()
    }

    pub fn decide(&self, generation: Power, consumption: Power) -> Power {
        generation - consumption
    }
}
