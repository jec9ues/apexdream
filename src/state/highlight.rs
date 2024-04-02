use std::mem::size_of;

use crate::sdk::{HighlightBits, HighlightSetting};

use super::*;

pub struct HighlightSettings {
    highlight_setting_ptr: sdk::Ptr<[HighlightSetting]>,
    settings: Box<[HighlightSetting]>,
}

impl Default for HighlightSettings {
    fn default() -> HighlightSettings {
        HighlightSettings {
            highlight_setting_ptr: sdk::Ptr::default(),
            settings: vec![HighlightSetting::default(); 255].into_boxed_slice(),
        }
    }
}

impl HighlightSettings {
    pub fn update(&mut self, api: &mut Api, ctx: &UpdateContext) {
        if !ctx.ticked(25, 14) {
            return;
        }

        let _ = api.vm_read_into(ctx.process.base.field(ctx.data.highlight_settings), &mut self.highlight_setting_ptr);
        if !self.highlight_setting_ptr.is_null() {
            let _ = api.vm_read_into(self.highlight_setting_ptr, &mut *self.settings);
        }
    }

    pub fn get(&self, index: usize) -> Option<HighlightSetting> {
        self.settings.get(index).copied()
    }

    pub fn set(&self, api: &mut Api, highlight_id: usize, bits: HighlightBits, color: Option<[f32; 3]>) {
        let _ = api.vm_write::<HighlightBits>(self.highlight_setting_ptr.field((size_of::<HighlightSetting>() * highlight_id) as u32), &bits);
        let Some(color) = color else { return; };
        let _ = api.vm_write::<[f32; 3]>(self.highlight_setting_ptr.field((size_of::<HighlightSetting>() * highlight_id + size_of::<HighlightBits>()) as u32), &color);
    }
}
