use nalgebra::Point3;

use crate::*;
use crate::base::solver::{LinearPredictor, solve, TargetPredictor};
use crate::cheats::config::{ESPConfig, LootESPConfig};
use crate::sdk::Character;

use super::espdata::{self, Flags, Object, SendObject};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Fade {
    Never,
    Far,
    Normal,
    Near,
}

impl Default for Fade {
    fn default() -> Fade {
        Fade::Normal
    }
}

pub struct Config {
    pub enable: bool,
    pub team: bool,
    pub debug_ents: bool,
    pub debug_bones: bool,
    pub debug_loot: bool,
    pub debug_models: bool,
    pub debug_local: bool,
    pub aim_bone1: u32,
    pub aim_bone2: u32,
    pub distance: f32,
    pub trail_fade: f32,
    pub flags_player: Flags,
    pub flags_downed: Flags,
    pub flags_npc: Flags,
    pub flags_deathbox: Flags,
    pub flags_loot: Flags,
    pub flags_anim: Flags,
    pub flags_vehicle: Flags,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            enable: true,
            team: false,
            aim_bone1: 11,
            aim_bone2: 12,
            debug_ents: false,
            debug_bones: false,
            debug_loot: false,
            debug_models: false,
            debug_local: false,
            distance: 18000.0,
            trail_fade: 30.0,
            flags_player: Flags::BOUNDS | Flags::HEALTH | Flags::AIM | Flags::SKYDOT,
            flags_downed: Flags::NAME | Flags::BOUNDS | Flags::FADED,
            flags_npc: Flags::TEXT | Flags::AIM | Flags::SKYDOT | Flags::BOUNDS | Flags::NAME | Flags::HEALTH,
            flags_deathbox: Flags::TEXT,
            flags_loot: Flags::ICON | Flags::TEXT,
            flags_anim: Flags::TEXT,
            flags_vehicle: Flags::TEXT,
        }
    }
}

#[derive(Default)]
pub struct ESP {
    config: Config,
    config2: ESPConfig,
}

impl ESP {
    #[inline(never)]
    pub fn render(&self,api: &mut Api, ctx: &RunContext) {
        if !self.config2.global.enable {
            return;
        }

        let Some(local) = ctx.state.local_player() else {
            return;
        };

        // Precalculate the set of desired items
        // Only show desired items if no active weapon or if weapon is melee
        let desired_items = if ctx.state.player_is_melee(local) {
            ctx.state.desired_items(local)
        } else {
            sdk::ItemSet::default()
        };

        let ref mut objects = Vec::new();
        for entity in ctx.state.entities() {
            match entity.as_ref() {
                EntityRef::Player(player) => self.player(ctx, objects, local, player),
                EntityRef::BaseNPC(npc) => self.npc(ctx, objects, local, npc),
                EntityRef::Deathbox(deathbox) => self.deathbox(ctx, objects, local, deathbox),
                EntityRef::Loot(loot) => self.loot(ctx, objects, local, loot, &desired_items),
                EntityRef::Animating(anim) => self.animating(ctx, objects, local, anim),
                EntityRef::Vehicle(vehicle) => self.vehicle(ctx, objects, local, vehicle),
                _ => (),
            }
        }

        // Sort back-to-front
        // Object struct is large, sort references instead
        let mut objects: Vec<&Object> = objects.iter().collect();
        objects.sort_unstable_by(|&lhs, &rhs| f32::total_cmp(&rhs.distance, &lhs.distance));
        let send_objects: Vec<SendObject> = objects.iter().map(|&i| {
            SendObject {
                flags: i.flags,
                name: if let Some(name) = i.name {Some(name.to_string())} else { None },
                text: if let Some(name) = i.text {Some(name.to_string())} else { None },
                visible: i.visible,
                color: i.color,
                fade_dist: i.fade_dist,
                alpha: i.alpha,
                origin: i.origin,
                view: i.view,
                spine: i.spine,
                aim: i.aim,
                skynade_pitch: i.skynade_pitch,
                skynade_yaw: i.skynade_yaw,
                distance: i.distance,
                width: i.width,
                height: i.height,
                health: i.health,
                max_health: i.max_health,
                shields: i.shields,
                max_shields: i.max_shields,
                model_name: if let Some(name) = i.model_name {Some(name.to_string())} else { None },
                skin: i.skin,
            }}).collect();
        api.update_objects(send_objects, ctx.state.client.view_matrix.clone(), ctx.screen.clone(), ctx.camera_origin().clone());
        let ref conf = espdata::Config {
            debug_bones: self.config.debug_bones,
            debug_models: self.config.debug_models,
            fade_distance: self.config.distance,
            fade_factor: 10.0,
            icon_grid: 47.0,
            icon_image: 0x76b36395,
            bounds_scale: 1.5,
            bounds_trans: 5.0,
        };
        // for object in objects {
        //     object.draw(api, ctx, conf);
        // }
    } /*pub fn render(&mut self, api: &mut Api, ctx: &RunContext) {
          if !self.config.enable {
              return;
          }

          let Some(local) = ctx.state.local_player() else { return };

          // Precalculate the set of desired items
          // Only show desired items if no active weapon or if weapon is melee
          let desired_items =
              if ctx.state.player_is_melee(local) { ctx.state.desired_items(local) }
              else { sdk::ItemSet::default() };

          let ref mut objects = Vec::new();
          for entity in ctx.state.entities() {
              match entity.as_ref() {
                  EntityRef::Player(player) => self.player(api, ctx, objects, local, player),
                  EntityRef::BaseNPC(npc) => self.npc(api, ctx, objects, local, npc),
                  EntityRef::Deathbox(deathbox) => self.deathbox(api, ctx, objects, local, deathbox),
                  EntityRef::Loot(loot) => self.loot(api, ctx, objects, local, loot, &desired_items),
                  EntityRef::Animating(anim) => self.animating(api, ctx, objects, local, anim),
                  EntityRef::Vehicle(vehicle) => self.vehicle(api, ctx, objects, local, vehicle),
                  _ => (),
              }
          }

          // Sort back-to-front
          // Object struct is large, sort references instead
          let mut objects: Vec<&Object> = objects.iter().collect();
          objects.sort_unstable_by(|&lhs, &rhs| f32::total_cmp(&rhs.distance, &lhs.distance));

          let ref conf = espdata::Config {
              debug_bones: self.config.debug_bones,
              debug_models: self.config.debug_models,
              fade_distance: self.config.distance,
              fade_factor: 10.0,
              icon_grid: 47.0,
              icon_image: 0x76b36395,
              bounds_scale: 1.5,
              bounds_trans: 5.0,
          };
          for object in objects {
              object.draw(api, ctx, conf);
          }
      }*/
}

/// Transform entities into ESP objects.
impl ESP {
    fn player<'a>(
        &self,
        ctx: &RunContext<'a>,
        objects: &mut Vec<Object<'a>>,
        local: &'a PlayerEntity,
        player: &'a PlayerEntity,
    ) {
        let config = &self.config2.player;

        if !config.enable {
            return;
        }
        if !player.is_alive() {
            return;
        }
        // out of config distance
        let distance = sdk::dist(local.origin, player.origin);
        if config.distance < distance {
            return;
        }
        // check debug local and teammate
        if ctx.state.is_same_team(local, player) {
            if local.index == player.index {
                if !config.debug_local {
                    return;
                }
            } else if !config.team {
                return;
            }
        }

        let (skynade_pitch, skynade_yaw) = skynade_angle(ctx.state, local, &player.origin);

        let va = if player.index == local.index {
            &local.view_angles
        } else {
            &player.angles
        };
        let target = LinearPredictor {
            origin: sdk::add(
                player.origin,
                player.bones.get_pos(self.config.aim_bone1 as usize),
            ),
            velocity: player.velocity,
        };

        objects.push(Object {
            name: ctx.state.get_player_name(player.get_info().handle),
            color: player.team_color.into(),
            visible: player.is_visible,
            fade_dist: self.get_fade(ctx.state, local, Fade::Normal),
            flags: if player.is_downed() {
                self.config.flags_downed
            } else {
                self.config.flags_player
            },
            origin: player.origin,
            distance: sdk::dist(player.origin, local.origin),
            bones: Some(&player.bones.v),
            studio: Some(&player.studio),
            spine: [
                player.bones.get_pos(self.config.aim_bone1 as usize),
                player.bones.get_pos(self.config.aim_bone2 as usize),
            ],
            aim: self.aim(ctx.state, local, &target),
            skynade_pitch,
            skynade_yaw,
            view: sdk::qvec(*va),
            trail: Some(&player.trail),
            width: 36.0,
            height: player.height(),
            health: player.health,
            max_health: player.max_health,
            shields: player.shields,
            max_shields: player.max_shields,
            model_name: Some(&player.model_name.string),
            ..Object::default()
        });
        let character = Character::get_by_model_name(&player.model_name.string);
        let hitbox_map = crate::state::studio::HitboxMap::get_by_model_name(&character);
        let bones = player.studio.get_player_bones(&Point3::from(player.origin), &player.bones, &hitbox_map);
        let mut hitbox_nodes = crate::state::studio::HitboxNodes::default();
        hitbox_nodes.update(&bones);

        // let lines = hitbox_nodes.get_pos();
        // for ([pos1, pos2], color) in lines {
        //     if let Some([x1, y1]) = ctx.world_to_screen(pos1, true) {
        //         if let Some([x2, y2]) = ctx.world_to_screen(pos2, true) {
        //             api.r_line(color, x1, y1, x2, y2)
        //         }
        //     }
        // }

        // let lines = &player.studio.hitboxes;
        // for index in 0..25 {
        //     let Some(studio) = lines.get(index) else { continue };
        //     let pos = player.bones.get_pos(studio.bone as usize);
        //     let pos = sdk::add(player.origin, pos);
        //     if let Some([x1, y1]) = ctx.world_to_screen(pos, true) {
        //             api.r_text(
        //                 /*font:*/ 0,
        //                 /*flags:*/ 3,
        //                 x1 + 10.0, y1 - 10.0,
        //                 /*width:*/ 1000.0,
        //                 /*height:*/ 100.0,
        //                 /*color*/ vgc::sRGBA!(Yellow),
        //                 /*shadow:*/ vgc::sRGBA!(Black),
        //                 /*text:*/ &fmtools::format!({index}"")
        //             );
        //     }
        // }

        // let bones = &player.bones;
        // for index in 0..100 {
        //     let pos = sdk::add(player.origin, player.bones.get_pos(index));
        //     if let Some([x1, y1]) = ctx.world_to_screen(pos, true) {
        //         api.r_text(
        //             /*font:*/ 0,
        //             /*flags:*/ 3,
        //             x1 + 20.0, y1 - 10.0,
        //             /*width:*/ 1000.0,
        //             /*height:*/ 100.0,
        //             /*color*/ vgc::sRGBA!(White),
        //             /*shadow:*/ vgc::sRGBA!(Black),
        //             /*text:*/ &fmtools::format!({index}"")
        //         );
        //     }
        // }
        // if let Some([x1, y1]) = ctx.world_to_screen(player.origin, true) {
        //     api.r_text(
        //         /*font:*/ 0,
        //         /*flags:*/ 3,
        //         x1 + 100.0, y1 - 100.0,
        //         /*width:*/ 1000.0,
        //         /*height:*/ 100.0,
        //         /*color*/ vgc::sRGBA!(Yellow),
        //         /*shadow:*/ vgc::sRGBA!(Black),
        //         /*text:*/ &fmtools::format!({player.model_name.string}"")
        //     );
        // }

        // if let Some([x1, y1]) = ctx.world_to_screen(player.origin, true) {
        //     api.r_text(
        //         /*font:*/ 0,
        //         /*flags:*/ 3,
        //         x1 + 200.0, y1 - 200.0,
        //         /*width:*/ 1000.0,
        //         /*height:*/ 100.0,
        //         /*color*/ vgc::sRGBA!(Yellow),
        //         /*shadow:*/ vgc::sRGBA!(Black),
        //         /*text:*/ &fmtools::format!({character:?}"")
        //     );
        // }
    }

    fn npc<'a>(
        &self,
        ctx: &RunContext<'a>,
        objects: &mut Vec<Object<'a>>,
        local: &'a PlayerEntity,
        npc: &'a BaseNPCEntity,
    ) {
        let config = &self.config2.npc;

        if !config.enable {
            return;
        }
        // out of config distance
        let distance = sdk::dist(local.origin, npc.origin);
        if config.distance < distance {
            return;
        }

        let is_melee = ctx.state.player_is_melee(local);
        let (class, fade) = match npc.model_name.hash {
            sdk::ModelName::LOOT_TICK if is_melee => {
                if npc.skin == 0 {
                    (strpool!(ctx, "tick"), Fade::Normal)
                } else {
                    (strpool!(ctx, "TICK"), Fade::Far)
                }
            }
            sdk::ModelName::MARVIN_BASE if is_melee => {
                if npc.skin == 0 {
                    (strpool!(ctx, "mrvn"), Fade::Normal)
                } else {
                    (strpool!(ctx, "MRVN"), Fade::Far)
                }
            }
            sdk::ModelName::PROWLER => (strpool!(ctx, "Prowler"), Fade::Normal),
            sdk::ModelName::DUMMIE_GENERIC
            | sdk::ModelName::DUMMIE_COMBAT
            | sdk::ModelName::DUMMIE_TRAINING => (strpool!(ctx, "DUMMIE"), Fade::Normal),
            sdk::ModelName::DUMMY => (strpool!(ctx, "DUMMY"), Fade::Normal),
            _ if !self.config.debug_models => return,
            _ => (strpool!(ctx, "BaseNPC"), Fade::Near),
        };

        let target = LinearPredictor {
            origin: sdk::add(
                npc.origin,
                npc.bones.get_pos(self.config.aim_bone1 as usize),
            ),
            velocity: [0.0; 3],
        };

        // let hitbox_map = crate::state::studio::HitboxMap::get_by_model_name(&Character::get_by_model_name(&npc.model_name.string));
        // let bones = npc.studio.get_player_bones(&Point3::from(npc.origin), &npc.bones, &hitbox_map);
        // let mut hitbox_nodes = crate::state::studio::HitboxNodes::default();
        // hitbox_nodes.update(&bones);
        //
        // let lines = hitbox_nodes.get_pos();
        // for ([pos1, pos2], color) in lines {
        //     if let Some([x1, y1]) = ctx.world_to_screen(pos1, true) {
        //         if let Some([x2, y2]) = ctx.world_to_screen(pos2, true) {
        //             api.r_line(color, x1, y1, x2, y2)
        //         }
        //     }
        // }

        let (skynade_pitch, skynade_yaw) = if ctx.state.is_firing_range() {
            skynade_angle(ctx.state, local, &npc.origin)
        } else {
            (0.0, 0.0)
        };

        objects.push(Object {
            text: Some(class),
            color: vgc::sRGB::White,
            visible: npc.is_visible,
            fade_dist: self.get_fade(ctx.state, local, fade),
            flags: self.config.flags_npc,
            origin: npc.origin,
            width: 36.0,
            height: 72.0,
            distance: sdk::dist(local.origin, npc.origin),
            bones: Some(&npc.bones.v),
            studio: Some(&npc.studio),
            spine: [
                npc.bones.get_pos(self.config.aim_bone1 as usize),
                npc.bones.get_pos(self.config.aim_bone2 as usize),
            ],
            aim: self.aim(ctx.state, local, &target),
            skynade_pitch,
            skynade_yaw,
            model_name: Some(&npc.model_name.string),
            skin: npc.skin,
            ..Object::default()
        });
    }

    fn deathbox<'a>(
        &self,
        ctx: &RunContext<'a>,
        objects: &mut Vec<Object<'a>>,
        local: &'a PlayerEntity,
        deathbox: &'a DeathboxEntity,
    ) {
        let config = &self.config2.deathbox;

        if !config.enable {
            return;
        }
        // out of config distance
        let distance = sdk::dist(local.origin, deathbox.origin);
        if config.distance < distance {
            return;
        }
        // Only show deathboxes while holding out melee
        if !ctx.state.player_is_melee(local) {
            return;
        }

        objects.push(Object {
            text: Some(strpool!(ctx, "LOOT")),
            color: vgc::sRGB::White,
            fade_dist: self.get_fade(ctx.state, local, Fade::Near),
            flags: self.config.flags_deathbox,
            origin: deathbox.origin,
            distance: sdk::dist(local.origin, deathbox.origin),
            ..Object::default()
        });
    }

    fn loot<'a>(
        &self,
        ctx: &RunContext<'a>,
        objects: &mut Vec<Object<'a>>,
        local: &'a PlayerEntity,
        loot: &'a LootEntity,
        desired_items: &sdk::ItemSet,
    ) {
        let config = &self.config2.loot;

        if !config.enable {
            return;
        }
        // Loot in deathboxes show up at the world origin
        if loot.origin == [0.0; 3] {
            return;
        }
        // out of config distance
        let distance = sdk::dist(local.origin, loot.origin);
        if config.distance < distance {
            return;
        }
        // Special rules for ground loot weapons
        if !config.debug && loot.weapon_name_index != 255 {
            // If the player doesn't have 2 weapons, add ESP for ground loot weapons
            if !(local.weapons[0].is_valid() && local.weapons[1].is_valid())
                && ctx.state.player_is_melee(local)
            {
                objects.push(Object {
                    text: match loot.weapon_name {
                        sdk::WeaponName::MASTIFF => Some(strpool!(ctx, "Mastiff")),
                        sdk::WeaponName::PEACEKEEPER => Some(strpool!(ctx, "PK")),
                        sdk::WeaponName::R301 => Some(strpool!(ctx, "R301")),
                        sdk::WeaponName::SENTINEL => Some(strpool!(ctx, "Sentinel")),
                        sdk::WeaponName::FLATLINE => Some(strpool!(ctx, "Flatline")),
                        sdk::WeaponName::WINGMAN => Some(strpool!(ctx, "Wingman")),
                        sdk::WeaponName::PROWLER => Some(strpool!(ctx, "Prowler")),
                        sdk::WeaponName::KRABER => Some(strpool!(ctx, "Kraber")),
                        sdk::WeaponName::G7_SCOUT => Some(strpool!(ctx, "Scout")),
                        sdk::WeaponName::BOCEK => Some(strpool!(ctx, "Bocek")),
                        sdk::WeaponName::RAMPAGE => Some(strpool!(ctx, "Rampage")),
                        sdk::WeaponName::RE45 => Some(strpool!(ctx, "RE45")),
                        sdk::WeaponName::NEMESIS => Some(strpool!(ctx, "Nemesis")),
                        sdk::WeaponName::HEMLOK => Some(strpool!(ctx, "Hemlok")),
                        sdk::WeaponName::EVA8_AUTO => Some(strpool!(ctx, "EVA8")),
                        sdk::WeaponName::CAR => Some(strpool!(ctx, "C.A.R")),
                        sdk::WeaponName::R99 => Some(strpool!(ctx, "R99")),
                        sdk::WeaponName::HAVOC => Some(strpool!(ctx, "Havoc")),
                        sdk::WeaponName::VOLT => Some(strpool!(ctx, "Volt")),
                        _ => None,
                    },
                    color: vgc::sRGB::White,
                    fade_dist: self.get_fade(ctx.state, local, Fade::Never),
                    flags: Flags::TEXT,
                    origin: loot.origin,
                    distance,
                    ..Object::default()
                });
            }
            return;
        }

        // Debug missing loot
        if config.debug || matches!(loot.known_item, sdk::ItemId::None) {
            let debug_print = fmtools::format!(
				"script int: "{loot.custom_script_int}"\n"
				"model name: "{loot.model_name.string}"\n"
				"bits: "{loot.bits.to_int():#010x}"\n"
				{loot.skin}" "{loot.skin_mod}" "{loot.body}" "{loot.camo_index});
            let debug_print = ctx.pool.store(debug_print);
            objects.push(Object {
                text: Some(debug_print),
                color: vgc::sRGB::White,
                fade_dist: self.get_fade(ctx.state, local, Fade::Never),
                flags: Flags::TEXT,
                origin: loot.origin,
                distance,
                model_name: Some(&loot.model_name.string),
                skin: loot.skin,
                ..Object::default()
            });
            return;
        }

        // Draw loot icon
        let index = loot.known_item.0 as usize;

        if index < desired_items.bit_len() && desired_items.bit_test(index) {
            // if true {
            objects.push(Object {
                color: vgc::sRGB::White,
                fade_dist: self.get_fade(ctx.state, local, Fade::Never),
                flags: self.config.flags_loot,
                origin: loot.origin,
                distance,
                text: Some(ctx.pool.store(fmtools::format!({loot.known_item}""))),
                icon: super::Icon::find(loot.known_item).map(|icon| espdata::Icon {
                    x: icon.gridx,
                    y: icon.gridy,
                }),
                ..Object::default()
            });
        }
    }

    fn animating<'a>(
        &self,
        ctx: &RunContext<'a>,
        objects: &mut Vec<Object<'a>>,
        local: &'a PlayerEntity,
        anim: &'a AnimatingEntity,
    ) {
        let config = &self.config2.animating;

        if !config.enable {
            return;
        }

        // out of config distance
        let distance = sdk::dist(local.origin, anim.origin);
        if config.distance < distance {
            return;
        }

        let (class, fade) = match anim.model_name.hash {
            sdk::ModelName::LOOT_SPHERE => (strpool!(ctx, "OO"), Fade::Normal),
            // sdk::ModelName::LOOT_BIN if ctx.state.player_is_melee(local) => {
            //     if anim.skin == 0 {
            //         (strpool!(ctx, "bin"), Fade::Near)
            //     } else {
            //         (strpool!(ctx, "BIN"), Fade::Near)
            //     }
            // }
            sdk::ModelName::GAS_TANK => (strpool!(ctx, "GAS"), Fade::Normal),
            sdk::ModelName::JUMP_PAD => (strpool!(ctx, "PAD"), Fade::Normal),
            // sdk::ModelName::TOTEM => (strpool!(ctx, "TOTEM"), Fade::Normal), removed ult
            sdk::ModelName::PARIAH_DRONE => (strpool!(ctx, "SEER"), Fade::Normal),
            sdk::ModelName::TROPHY_SYSTEM => (strpool!(ctx, "PYLON"), Fade::Normal),
            sdk::ModelName::CONDUIT_SHIELD_JAMMER => (ctx.pool.store(fmtools::format!("JAMMER")), Fade::Normal),
            sdk::ModelName::MACHINE_GUN => (ctx.pool.store(fmtools::format!("MACHINE GUN")), Fade::Normal),
            // Draw a count down timer for the gibraltar dome shield
            sdk::ModelName::BUBBLESHIELD => {
                const DOME_TIME: f64 = 12.0 + 0.5; // 12s + 0.5s activation
                let timer = DOME_TIME - (ctx.state.time - anim.spawn_time);
                if timer < 0.0 {
                    return;
                }
                (
                    ctx.pool.store(fmtools::format!({timer:.1}" s")),
                    Fade::Normal,
                )
            }
            sdk::ModelName::EMPTY_MODEL => return,
            _ if !config.debug => return,
            _ => (ctx.pool.store(fmtools::format!({anim.model_name.string}"")), Fade::Near),
        };

        objects.push(Object {
            text: Some(class),
            color: vgc::sRGB::Red,
            fade_dist: self.get_fade(ctx.state, local, fade),
            flags: self.config.flags_anim,
            origin: anim.origin,
            distance: sdk::dist(local.origin, anim.origin),
            model_name: Some(&anim.model_name.string),
            skin: anim.skin,
            ..Object::default()
        });
    }

    fn vehicle<'a>(
        &self,
        ctx: &RunContext<'a>,
        objects: &mut Vec<Object<'a>>,
        local: &'a PlayerEntity,
        vehicle: &'a VehicleEntity,
    ) {
        let config = &self.config2.vehicle;

        if !config.enable {
            return;
        }

        // out of config distance
        let distance = sdk::dist(local.origin, vehicle.origin);
        if config.distance < distance {
            return;
        }

        objects.push(Object {
            text: Some(strpool!(ctx, "^.^")),
            color: ctx
                .state
                .entity_as::<PlayerEntity>(vehicle.vehicle_driver)
                .map(|driver| driver.team_color.into())
                .unwrap_or(vgc::sRGB::White),
            fade_dist: self.get_fade(ctx.state, local, Fade::Normal),
            flags: self.config.flags_vehicle,
            origin: vehicle.origin,
            distance: sdk::dist(local.origin, vehicle.origin),
            ..Object::default()
        });
    }

    fn aim(
        &self,
        state: &GameState,
        local: &PlayerEntity,
        target: &dyn TargetPredictor,
    ) -> Option<[f32; 3]> {
        let weapon = local.active_weapon(state)?;

        if weapon.projectile_speed <= 1.0 {
            return None;
        }

        let sol = solve(&local.view_origin, weapon, target)?;

        Some([-sol.pitch.to_degrees(), sol.yaw.to_degrees(), 0.0])
    }

    fn get_fade(&self, state: &GameState, local: &PlayerEntity, fade: Fade) -> f32 {
        // Factor in the zoom level to increase its effective range
        use std::f32::consts::TAU;
        let zoom = f32::tan(TAU / 8.0) / f32::tan(state.get_fov(local).to_radians() / 2.0);

        // If we're skydiving or in the dropship draw everything at all ranges
        // This gives us a good overview of the whole map
        let max_dist = match fade {
            Fade::Near => self.config.distance * 0.25,
            _ if false || local.camera_origin[2] > sdk::AIRSHIP_HEIGHT => return 1000000.0,
            Fade::Normal => self.config.distance,
            Fade::Far => self.config.distance * 8.0,
            Fade::Never => return 1000000.0,
        };

        return max_dist * zoom;
    }
}

fn skynade_angle(state: &GameState, local: &PlayerEntity, target: &[f32; 3]) -> (f32, f32) {
    let Some(weapon) = local.active_weapon(state) else {
        return Default::default();
    };

    let (lob, pitches, z_offset): (bool, &[sdk::pitches::Pitch], f32) =
        match (weapon.mod_bitfield & 0x4 != 0, weapon.weapon_name) {
            (false, sdk::WeaponName::THERMITE_GRENADE) => {
                (false, &sdk::pitches::GRENADE_PITCHES, 0.0)
            }
            (false, sdk::WeaponName::FRAG_GRENADE) => (true, &sdk::pitches::GRENADE_PITCHES, 70.0),
            (false, sdk::WeaponName::ARC_STAR) => (false, &sdk::pitches::ARC_PITCHES, 25.0),
            (true, sdk::WeaponName::THERMITE_GRENADE) => {
                (false, &sdk::pitches::GRENADIER_GRENADE_PITCHES, 0.0)
            }
            (true, sdk::WeaponName::FRAG_GRENADE) => {
                (false, &sdk::pitches::GRENADIER_GRENADE_PITCHES, 70.0)
            }
            (true, sdk::WeaponName::ARC_STAR) => {
                (false, &sdk::pitches::GRENADIER_ARC_PITCHES, 25.0)
            }
            _ => return Default::default(),
        };

    let g = 750.0 * weapon.projectile_scale;
    let v0 = weapon.projectile_speed;

    let delta = sdk::sub(*target, local.view_origin);
    let delta = sdk::add(delta, sdk::muls(delta, 20.0 / sdk::len(delta)));
    let dx = f32::sqrt(delta[0] * delta[0] + delta[1] * delta[1]);
    let dy = delta[2] + z_offset;

    let calc_angle = if lob { lob_angle } else { optimal_angle };
    if let Some(launch_pitch) = calc_angle(dx, dy, v0, g) {
        let view_pitch = sdk::pitches::launch2view(pitches, launch_pitch);
        return (
            view_pitch,
            sdk::qangle(sdk::sub(*target, local.view_origin))[1].to_radians(),
        );
    } else {
        return Default::default();
    }

    fn optimal_angle(x: f32, y: f32, v0: f32, g: f32) -> Option<f32> {
        let root = v0 * v0 * v0 * v0 - g * (g * x * x + 2.0 * y * v0 * v0);
        if root < 0.0 {
            return None;
        }
        let root = f32::sqrt(root);
        let slope = (v0 * v0 - root) / (g * x);
        Some(f32::atan(slope))
    }
    fn lob_angle(x: f32, y: f32, v0: f32, g: f32) -> Option<f32> {
        let root = v0 * v0 * v0 * v0 - g * (g * x * x + 2.0 * y * v0 * v0);
        if root < 0.0 {
            return None;
        }
        let root = f32::sqrt(root);
        let slope = (v0 * v0 + root) / (g * x);
        Some(f32::atan(slope))
    }
}
