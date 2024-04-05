use super::*;

#[derive(Default)]
pub struct LootEntity {
    pub entity_ptr: sdk::Ptr,
    pub entity_size: u32,
    pub index: u32,
    pub origin: [f32; 3],

    update_rate: u32,
    update_time: f64,

    pub model_name: ModelName,

    pub skin: i32,
    pub skin_mod: i32,
    pub body: i32,
    pub camo_index: i32,

    // 17 season new parameter
    pub highlight_fix: u8,
    pub highlight_type: u8,
    pub highlight_id: u8,
    pub color: [f32; 3],
    pub bits: sdk::HighlightBits,

    pub ammo_in_clip: i32,
    pub custom_script_int: i32,
    pub known_item: sdk::ItemId,
    pub survival_property: i32,
    pub weapon_name_index: i32,
    pub weapon_name: sdk::WeaponName,
    pub mod_bitfield: u32,
}

impl LootEntity {
    pub fn new(entity_ptr: sdk::Ptr, index: u32, cc: &sdk::ClientClass) -> Box<dyn Entity> {
        let entity_size = cc.ClassSize;
        Box::new(LootEntity {
            entity_ptr,
            entity_size,
            index,
            ..LootEntity::default()
        }) as Box<dyn Entity>
    }
}
#[derive(Default, Serialize, Deserialize)]
pub struct NetLootEntity {
    pub index: u32,
    pub origin: [f32; 3],
    pub model_name: String,

    pub custom_script_int: i32,
    pub known_item: String,
    pub weapon_name_index: i32,
    pub weapon_name: String,
}
impl Entity for LootEntity {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_ref(&self) -> EntityRef<'_> {
        EntityRef::Loot(self)
    }
    fn is_serialized(&self) -> bool {
        true
    }
    fn get_info(&self) -> EntityInfo {
        EntityInfo {
            entity_ptr: self.entity_ptr,
            index: self.index as usize,
            handle: sdk::EHandle::from(self.index),
            rate: self.update_rate,
        }
    }
    fn get_json(&self, game_state: &GameState) -> Option<NetEntity> {
        Some(NetEntity::Loot(NetLootEntity {
            index: self.index,
            origin: self.origin,
            model_name: self.model_name.string.clone(),
            custom_script_int: self.custom_script_int,
            known_item: self.known_item.to_string(),
            weapon_name_index: self.weapon_name_index,
            weapon_name: self.weapon_name.to_string(),
        }))
    }
    fn update(&mut self, api: &mut Api, ctx: &UpdateContext) {
        #[derive(sdk::Pod)]
        #[repr(C)]
        struct Indices {
            origin: [u32; 3],
            model_name: [u32; 2],
            skin: [u32; 4],
            highlight: [u32; 3],
            survival: [u32; 5],
        }

        let data = ctx.data;
        let mut indices = Indices {
            origin: [
                data.entity_origin + 0,
                data.entity_origin + 4,
                data.entity_origin + 8,
            ],
            model_name: [data.entity_model_name + 0, data.entity_model_name + 4],
            skin: [
                data.animating_skin + 0,
                data.animating_skin + 4,
                data.animating_skin + 8,
                data.animating_skin + 12,
            ],
            highlight: [
                0x268, /*GLOW_FIX*/
                0x26c, /*GLOW_VISIBLE_TYPE*/
                0x28c, /*GLOW_HIGHLIGHT_ID*/
            ],
            // highlight: [
            // 	data.entity_highlight + 0,
            // 	data.entity_highlight + 4,
            // 	data.entity_highlight + 8,
            // 	data.entity_highlight + 4 * 3 * 2 * sdk::HIGHLIGHT_MAX_CONTEXTS as u32],
            survival: [
                data.prop_survival + 0,
                data.prop_survival + 4,
                data.prop_survival + 8,
                data.prop_survival + 16,
                data.prop_survival + 20,
            ],
        };

        if let Ok(fields) = api.vm_gatherd(self.entity_ptr, self.entity_size, &mut indices) {
            let origin = [
                f32::from_bits(fields.origin[0]),
                f32::from_bits(fields.origin[1]),
                f32::from_bits(fields.origin[2]),
            ];
            if self.origin != origin {
                self.update_time = ctx.time;
            }
            self.origin = origin;

            let model_name_ptr = fields.model_name[0] as u64 | (fields.model_name[1] as u64) << 32;
            self.model_name.update(api, model_name_ptr.into());

            self.skin = fields.skin[0] as i32;
            self.skin_mod = fields.skin[1] as i32;
            self.body = fields.skin[2] as i32;
            self.camo_index = fields.skin[3] as i32;

            self.highlight_fix = fields.highlight[0] as u8;
            self.highlight_type = fields.highlight[1] as u8;
            self.highlight_id = fields.highlight[2] as u8;

            // let Ok(tmp_highlight_fix) = api.vm_read(self.entity_ptr.field::<u8>(0x268)) else { return; };
            // let Ok(tmp_highlight_type) = api.vm_read(self.entity_ptr.field::<u8>(0x26c)) else { return; };
            // let Ok(tmp_highlight_id) = api.vm_read(self.entity_ptr.field::<u8>(0x28d)) else { return; };
            //
            // assert_eq!(self.highlight_fix, tmp_highlight_fix);
            // assert_eq!(self.highlight_type, tmp_highlight_type);
            // assert_eq!(self.highlight_id, tmp_highlight_id);

            /*			let OFF_GLOW_HIGHLIGHTS: u32 = 0xb944e30;
            if let Ok(highlight_settings_ptr) = api.vm_read::<sdk::Ptr>(ctx.process.base.field(OFF_GLOW_HIGHLIGHTS) ) {
                if let Ok(context_id) = api.vm_read::<u8>(self.entity_ptr.field(0x28c)) {
                    if let Ok(mut highlight_setting) = api.vm_read::<HighlightSetting>(highlight_settings_ptr.field((size_of::<HighlightSetting>() * context_id as usize) as u32)) {
                        // println!("{:?} -> {:?}", context_id, highlight_settings.params.color);
                        self.color = highlight_setting.params.color;
                        self.bits = highlight_setting.bits;
                    }
                }
            }*/
            // self.color[0] = f32::from_bits(fields.highlight[0]);
            // self.color[1] = f32::from_bits(fields.highlight[1]);
            // self.color[2] = f32::from_bits(fields.highlight[2]);
            // self.bits = sdk::HighlightBits::from_uint(fields.highlight[3]);

            self.ammo_in_clip = fields.survival[0] as i32;
            self.custom_script_int = fields.survival[1] as i32;
            self.survival_property = fields.survival[2] as i32;
            self.weapon_name_index = fields.survival[3] as u16 as i32;
            self.mod_bitfield = fields.survival[4];
        }

        self.update_rate = if ctx.time >= self.update_time + 0.25 {
            512
        } else {
            2
        };
    }
    fn post(&mut self, _api: &mut Api, _ctx: &UpdateContext, state: &GameState) {
        self.weapon_name = state.weapon_name(self.weapon_name_index);
        self.known_item = state.known_item(self.custom_script_int);
        if let Some(highlightsetting) = state.highlight.get(self.highlight_id as usize) {
            self.color = highlightsetting.params.color;
            self.bits = highlightsetting.bits;
        }
    }
}
