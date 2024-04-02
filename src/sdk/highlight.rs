use std::fmt;

use obfstr::obfstr as s;

use super::Pod;

// https://www.unknowncheats.me/forum/apex-legends/446349-script-highlight.html

#[derive(Copy, Clone, Pod)]
#[repr(C)]
pub struct HighlightParams {
    pub color: [f32; 3],
    pub velocity: [f32; 3],
}

impl fmt::Debug for HighlightParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct(s!("HighlightParams"))
            .field(s!("color"), &format_args!("{:?}", self.color))
            .field(s!("velocity"), &format_args!("{:?}", self.velocity))
            .finish()
    }
}

#[derive(Copy, Clone, Default, Pod)]
#[repr(C)]
pub struct HighlightBits {
    pub inside_function: u8,
    pub outline_function: u8,
    pub outline_radius: u8,
    // 1.0..8.0
    pub inside_opacity: u8,
}

impl HighlightBits {
    pub const fn new(
        inside_function: u8,
        outline_function: u8,
        outline_radius: u8,
        inside_opacity: u8,
        is_entity_visible: bool,
        is_after_post_process: bool,
    ) -> HighlightBits {
        HighlightBits {
            inside_function,
            outline_function,
            outline_radius,
            inside_opacity: inside_opacity
                | if is_entity_visible { 0x40 } else { 0 }
                | if is_after_post_process { 0x80 } else { 0 },
        }
    }

    /// Bloodhound scan effect.
    pub const SONAR: HighlightBits = HighlightBits::new(12, 169, 32, 7, true, false);
}

impl HighlightBits {
    pub fn from_uint(int: u32) -> HighlightBits {
        unsafe { std::mem::transmute(int) }
    }
    pub fn to_int(&self) -> u32 {
        (self.inside_opacity as u32) << 24
            | (self.outline_radius as u32) << 16
            | (self.outline_function as u32) << 8
            | (self.inside_function as u32)
    }
    pub fn outline_radius(&self) -> f32 {
        self.outline_radius as f32 * (8.0 / 255.0)
    }
    pub fn is_entity_visible(&self) -> bool {
        self.inside_opacity & 0x40 != 0
    }
    pub fn is_after_post_process(&self) -> bool {
        self.inside_opacity & 0x80 != 0
    }
}

impl fmt::Debug for HighlightBits {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct(s!("HighlightBits"))
            .field(s!("inside_function"), &self.inside_function)
            .field(s!("outline_function"), &self.outline_function)
            .field(s!("outline_radius"), &self.outline_radius)
            .field(s!("inside_opacity"), &(self.inside_opacity & 0x3f))
            .field(s!("is_entity_visible"), &self.is_entity_visible())
            .field(s!("is_after_post_process"), &self.is_after_post_process())
            .finish()
    }
}

#[derive(Copy, Clone, Pod)]
#[repr(C)]
pub struct HighlightSetting {
    pub bits: HighlightBits,
    pub params: HighlightParams,
    pub pad_1c: [u8; 0x18],
}

impl fmt::Debug for HighlightSetting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct(s!("HighlightSetting"))
            .field(s!("bits"), &format_args!("{:?}", &self.bits))
            .field(s!("params"), &format_args!("{:?}", &self.params))
            .finish()
    }
}

impl Default for HighlightSetting {
    fn default() -> HighlightSetting {
        dataview::zeroed()
    }
}

pub const HIGHLIGHT_CONTEXT_NONE: i32 = -1;
pub const HIGHLIGHT_CONTEXT_NEUTRAL: i32 = 0;
pub const HIGHLIGHT_CONTEXT_FRIENDLY: i32 = 1;
pub const HIGHLIGHT_CONTEXT_ENEMY: i32 = 2;
pub const HIGHLIGHT_CONTEXT_OWNED: i32 = 3;
pub const HIGHLIGHT_CONTEXT_PRIVATE_MATCH_OBSERVER: i32 = 4;
pub const HIGHLIGHT_CHARACTER_SPECIAL_HIGHLIGHT: i32 = 5;
pub const HIGHLIGHT_CONTEXT_DEATH_RECAP: i32 = 6;
pub const HIGHLIGHT_CONTEXT_SONAR: i32 = 7;
pub const HIGHLIGHT_CHARACTER_SPECIAL_HIGHLIGHT_2: i32 = 8;
pub const HIGHLIGHT_CONTEXT_FRIENDLY_REVEALED: i32 = 9;
pub const HIGHLIGHT_CONTEXT_MOVEMENT_REVEALED: i32 = 10;
pub const HIGHLIGHT_MAX_CONTEXTS: usize = 11;

pub const HIGHLIGHT_VIS_NONE: i32 = 0;
pub const HIGHLIGHT_VIS_FORCE_ON: i32 = 1;
pub const HIGHLIGHT_VIS_ALWAYS: i32 = 2;
pub const HIGHLIGHT_VIS_OCCLUDED: i32 = 3;
pub const HIGHLIGHT_VIS_FULL_VIEW: i32 = 4;
pub const HIGHLIGHT_VIS_LOS: i32 = 5;
pub const HIGHLIGHT_VIS_LOS_ENTSONLYCONTENTSBLOCK: i32 = 6;

pub const HIGHLIGHT_FLAG_NONE: u32 = 0;
pub const HIGHLIGHT_FLAG_ADS_FADE: u32 = 1;
pub const HIGHLIGHT_FLAG_REQUIRE_NOT_FULL_HEALTH: u32 = 2;
pub const HIGHLIGHT_FLAG_REQUIRE_CAN_PICK_UP_CLIP: u32 = 4;
pub const HIGHLIGHT_FLAG_REQUIRE_CAN_PICK_UP_OFFHAND: u32 = 8;
pub const HIGHLIGHT_FLAG_REQUIRE_WEAKPOINT_VISIBLE: u32 = 16;
pub const HIGHLIGHT_FLAG_REQUIRE_PILOT: u32 = 32;
pub const HIGHLIGHT_FLAG_REQUIRE_TITAN: u32 = 64;
pub const HIGHLIGHT_FLAG_REQUIRE_SAME_TEAM: u32 = 128;
pub const HIGHLIGHT_FLAG_REQUIRE_DIFFERENT_TEAM: u32 = 256;
pub const HIGHLIGHT_FLAG_REQUIRE_FRIENDLY_TEAM: u32 = 512;
pub const HIGHLIGHT_FLAG_REQUIRE_ENEMY_TEAM: u32 = 1024;
pub const HIGHLIGHT_FLAG_REQUIRE_LOCAL_PLAYER_IS_OWNER: u32 = 2048;
pub const HIGHLIGHT_FLAG_REQUIRE_LOW_MOVEMENT: u32 = 4096;
pub const HIGHLIGHT_FLAG_REQUIRE_HIGH_MOVEMENT: u32 = 8192;
pub const HIGHLIGHT_FLAG_CHECK_OFTEN: u32 = 16384;
// HIGHLIGHT_FLAG_DISABLE_DEATH_FADE = _ImageBase, 32768
// HIGHLIGHT_FLAG_TEAM_AGNOSTIC = &loc_20000       65536
