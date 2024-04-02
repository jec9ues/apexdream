#[derive(Default)]
pub struct CheatConfig {
    pub highlight: HighlightConfigs,
    pub esp: ESPConfig,
}

#[derive(Default)]
pub struct HighlightConfigs {
    pub player: HighlightConfig,
}

#[derive(Default)]
struct HighlightConfig {
    pub enable: bool,
    pub debug: bool,
}

#[derive(Default)]
pub struct ESPConfig {
    pub global: GlobalESPConfig,
    pub npc: NPCESPConfig,
    pub player: PlayerESPConfig,
    pub deathbox: DeathboxESPConfig,
    pub loot: LootESPConfig,
    pub animating: AnimaingESPConfig,
    pub vehicle: VehicleESPConfig,

}

pub struct GlobalESPConfig {
    pub enable: bool,
}

impl Default for GlobalESPConfig {
    fn default() -> Self {
        Self {
            enable: true,
        }
    }
}

pub struct PlayerESPConfig {
    pub enable: bool,
    pub distance: f32,
    pub bounds: bool,
    pub name: bool,
    pub health: bool,
    pub skydot: bool,
    pub show_distance: bool,

    pub team: bool,
    pub debug_local: bool,
    pub debug_aim: bool,
    pub debug_bones: bool,
}

impl Default for PlayerESPConfig {
    fn default() -> Self {
        Self {
            enable: true,
            distance: 30000.0,
            bounds: false,
            name: false,
            health: false,
            skydot: false,
            show_distance: false,
            team: false,
            debug_local: false,
            debug_aim: false,
            debug_bones: false,
        }
    }
}

pub struct NPCESPConfig {
    pub enable: bool,
    pub debug: bool,
    pub distance: f32,
    pub show_distance: bool,
}

impl Default for NPCESPConfig {
    fn default() -> Self {
        Self {
            enable: true,
            debug: false,
            distance: 300000.0,
            show_distance: true,
        }
    }
}

pub struct DeathboxESPConfig {
    pub enable: bool,
    pub distance: f32,
    pub show_distance: bool,
}

impl Default for DeathboxESPConfig {
    fn default() -> Self {
        Self {
            enable: true,
            distance: 3000.0,
            show_distance: true,
        }
    }
}


pub struct LootESPConfig {
    pub enable: bool,
    pub debug: bool,
    pub distance: f32,
    pub show_distance: bool,
}

impl Default for LootESPConfig {
    fn default() -> Self {
        Self {
            enable: true,
            debug: false,
            distance: 3000.0,
            show_distance: true,
        }
    }
}

pub struct AnimaingESPConfig {
    pub enable: bool,
    pub debug: bool,
    pub distance: f32,
    pub show_distance: bool,
}

impl Default for AnimaingESPConfig {
    fn default() -> Self {
        Self {
            enable: true,
            debug: false,
            distance: 3000.0,
            show_distance: true,
        }
    }
}

pub struct VehicleESPConfig {
    pub enable: bool,
    pub distance: f32,
    pub show_distance: bool,
}

impl Default for VehicleESPConfig {
    fn default() -> Self {
        Self {
            enable: true,
            distance: 3000.0,
            show_distance: true,
        }
    }
}