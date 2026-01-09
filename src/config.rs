// SPDX-License-Identifier: MPL-2.0

use cosmic::cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry};

#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    pub work_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    pub work_sessions_per_cycle: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_minutes: 20,
            short_break_minutes: 5,
            long_break_minutes: 20,
            work_sessions_per_cycle: 3,
        }
    }
}
