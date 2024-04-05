#![allow(dead_code)]

use std::any::Any;
use serde::{Deserialize, Serialize};
use crate::state::entities::animating::NetAnimatingEntity;
use crate::state::entities::deathbox::NetDeathboxEntity;
use crate::state::entities::loot::NetLootEntity;
use crate::state::entities::npc::NetBaseNPCEntity;
use crate::state::entities::player::NetPlayerEntity;
use crate::state::entities::vehicle::NetVehicleEntity;
use crate::state::entities::weaponx::NetWeaponXEntity;
use crate::state::entities::world::NetWorldEntity;

use super::*;

pub use self::animating::AnimatingEntity;
pub use self::base::BaseEntity;
pub use self::deathbox::DeathboxEntity;
pub use self::loot::LootEntity;
pub use self::npc::BaseNPCEntity;
pub use self::player::PlayerEntity;
pub use self::projectile::ProjectileEntity;
pub use self::scriptnetdata::ScriptNetDataEntity;
pub use self::utils::BoneArray;
pub use self::vehicle::VehicleEntity;
pub use self::waypoint::WaypointEntity;
pub use self::weaponx::WeaponXEntity;
pub use self::world::WorldEntity;

#[derive(Copy, Clone, Debug, Default)]
pub struct EntityInfo {
    pub entity_ptr: sdk::Ptr,
    pub index: usize,
    pub handle: sdk::EHandle,
    pub rate: u32,
}

pub trait Entity: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_ref(&self) -> EntityRef<'_>;
    fn is_serialized(&self) -> bool;
    fn get_info(&self) -> EntityInfo;
    fn get_json(&self, game_state: &GameState) -> Option<NetEntity>;
    fn update(&mut self, api: &mut Api, ctx: &UpdateContext);
    fn post(&mut self, _api: &mut Api, _ctx: &UpdateContext, _state: &GameState) {}
}

#[derive(Serialize, Deserialize)]
pub enum NetEntity {
    BaseEntity(NetBaseEntity),
    BaseNPC(NetBaseNPCEntity),
    World(NetWorldEntity),
    Player(NetPlayerEntity),
    WeaponX(NetWeaponXEntity),
    Loot(NetLootEntity),
    Waypoint(NetWaypointEntity),
    Vehicle(NetVehicleEntity),
    Deathbox(NetDeathboxEntity),
    Animating(NetAnimatingEntity),
    Projectile(NetProjectileEntity),
    ScriptNetData(NetScriptNetDataEntity),
}

#[derive(Default, Serialize, Deserialize)]
pub struct NetBaseEntity {}

#[derive(Default, Serialize, Deserialize)]
pub struct NetWaypointEntity {}

#[derive(Default, Serialize, Deserialize)]
pub struct NetProjectileEntity {}

#[derive(Default, Serialize, Deserialize)]
pub struct NetScriptNetDataEntity {}


#[derive(Copy, Clone)]
pub enum EntityRef<'a> {
    BaseEntity(&'a BaseEntity),
    BaseNPC(&'a BaseNPCEntity),
    World(&'a WorldEntity),
    Player(&'a PlayerEntity),
    WeaponX(&'a WeaponXEntity),
    Loot(&'a LootEntity),
    Waypoint(&'a WaypointEntity),
    Vehicle(&'a VehicleEntity),
    Deathbox(&'a DeathboxEntity),
    Animating(&'a AnimatingEntity),
    Projectile(&'a ProjectileEntity),
    ScriptNetData(&'a ScriptNetDataEntity),
}

impl EntityRef<'_> {
    pub fn get_type_name(self, buf: &mut [u8; 32]) -> &str {
        match self {
            EntityRef::BaseEntity(_) => s!(buf <- "BaseEntity"),
            EntityRef::BaseNPC(_) => s!(buf <- "BaseNPC"),
            EntityRef::World(_) => s!(buf <- "World"),
            EntityRef::Player(_) => s!(buf <- "Player"),
            EntityRef::WeaponX(_) => s!(buf <- "WeaponX"),
            EntityRef::Loot(_) => s!(buf <- "Loot"),
            EntityRef::Waypoint(_) => s!(buf <- "Waypoint"),
            EntityRef::Vehicle(_) => s!(buf <- "Vehicle"),
            EntityRef::Deathbox(_) => s!(buf <- "Deathbox"),
            EntityRef::Animating(_) => s!(buf <- "Animating"),
            EntityRef::Projectile(_) => s!(buf <- "Projectile"),
            EntityRef::ScriptNetData(_) => s!(buf <- "ScriptNetData"),
        }
    }
}

mod animating;
mod base;
mod deathbox;
mod loot;
mod npc;
mod player;
mod projectile;
mod scriptnetdata;
mod vehicle;
mod waypoint;
mod weaponx;
mod world;

mod utils;

#[derive(Clone, Default)]
pub struct ModelName {
    pub ptr: sdk::Ptr<[u8]>,
    pub string: String,
    pub hash: sdk::ModelName,
}

impl ModelName {
    pub fn update(&mut self, api: &mut Api, model_name_ptr: sdk::Ptr<[u8]>) -> bool {
        // Update when pointer changes
        if model_name_ptr != self.ptr {
            self.string.clear();
            self.hash = Default::default();
            if !model_name_ptr.is_null() {
                let mut model_name = [0u8; 128];
                if let Ok(model_name) = api.vm_read_cstr(model_name_ptr, &mut model_name) {
                    self.string.push_str(model_name);
                    self.string.make_ascii_lowercase(); // Keep everything consistently lower cased
                    self.hash = sdk::ModelName(crate::hash(&self.string));
                    return true;
                }
            }
        }
        return false;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct HitSphere {
    pub bone: i32,
    pub radius: f32,
}


#[derive(Default, Serialize, Deserialize)]
pub struct NetData {
    pub entites: NetEntities,
}

impl NetData {
    pub fn update(&mut self, ctx: &RunContext) {
        self.entites.clear();
        self.entites.update(ctx.state)
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct NetEntities {
    pub level_name: String,
    pub local_player: Option<NetEntity>,
    pub npc: Vec<NetEntity>,
    pub world: Vec<NetEntity>,
    pub player: Vec<NetEntity>,
    pub animating: Vec<NetEntity>,
    pub weapon: Vec<NetEntity>,
    pub loot: Vec<NetEntity>,
    pub vehicle: Vec<NetEntity>,
    pub deathbox: Vec<NetEntity>,
}

impl NetEntities {
    pub fn clear(&mut self) {
        self.level_name.clear();
        self.local_player = None;
        self.npc.clear();
        self.world.clear();
        self.player.clear();
        self.animating.clear();
        self.weapon.clear();
        self.loot.clear();
        self.vehicle.clear();
        self.deathbox.clear();
    }
    pub fn update(&mut self, game_state: &GameState) {
        self.level_name = game_state.client.level_name.clone();
        self.local_player = if let Some(player) = game_state.local_player() { player.get_json(game_state) } else { None };
        for entity in game_state.entities() {
            match entity.as_ref() {
                EntityRef::BaseNPC(npc) => if let Some(json) = npc.get_json(game_state) { self.npc.push(json) },
                EntityRef::World(world) => if let Some(json) = world.get_json(game_state) { self.world.push(json) },
                EntityRef::Player(player) => if let Some(json) = player.get_json(game_state) { self.player.push(json) },
                EntityRef::WeaponX(weapon) => if let Some(json) = weapon.get_json(game_state) { self.weapon.push(json) },
                EntityRef::Loot(loot) => if let Some(json) = loot.get_json(game_state) { self.loot.push(json) },
                EntityRef::Vehicle(vehicle) => if let Some(json) = vehicle.get_json(game_state) { self.vehicle.push(json) },
                EntityRef::Deathbox(deathbox) => if let Some(json) = deathbox.get_json(game_state) { self.deathbox.push(json) },
                EntityRef::Animating(anim) => if let Some(json) = anim.get_json(game_state) { self.animating.push(json) },
                _ => (),
            }
        }
    }
}