use crate::*;

pub struct Icon {
    pub item: u16,
    pub gridx: u8,
    pub gridy: u8,
}

impl Icon {
    const fn new(_name: &[u8; 4], gridx: u8, gridy: u8, item: sdk::ItemId) -> Icon {
        Icon {
            item: item.0 as u16,
            gridx,
            gridy,
        }
    }
    pub fn item(&self) -> sdk::ItemId {
        sdk::ItemId(self.item as u16)
    }
    #[inline(never)]
    pub fn find(item: sdk::ItemId) -> Option<&'static Icon> {
        let index = ICONS.binary_search_by_key(&item, |ico| ico.item()).ok()?;
        Some(&ICONS[index])
    }
}

pub static ICONS: [Icon; 84] = [
    Icon::new(b"LiRo", 0, 0, sdk::ItemId::LightRounds),
    Icon::new(b"EnAm", 1, 0, sdk::ItemId::EnergyAmmo),
    Icon::new(b"ShSh", 2, 0, sdk::ItemId::ShotgunShells),
    Icon::new(b"HeRo", 3, 0, sdk::ItemId::HeavyRounds),
    Icon::new(b"SnAm", 4, 0, sdk::ItemId::SniperAmmo),
    Icon::new(b"Arws", 5, 0, sdk::ItemId::Arrows),
    Icon::new(b"UlAc", 0, 1, sdk::ItemId::UltAccel),
    Icon::new(b"Phoe", 1, 1, sdk::ItemId::PhoenixKit),
    Icon::new(b"MedK", 2, 1, sdk::ItemId::MedKit),
    Icon::new(b"Syri", 3, 1, sdk::ItemId::Syringe),
    Icon::new(b"Batt", 4, 1, sdk::ItemId::Battery),
    Icon::new(b"Cell", 5, 1, sdk::ItemId::ShieldCell),
    Icon::new(b"Hel1", 0, 2, sdk::ItemId::HelmetLv1),
    Icon::new(b"Hel2", 1, 2, sdk::ItemId::HelmetLv2),
    Icon::new(b"Hel3", 2, 2, sdk::ItemId::HelmetLv3),
    Icon::new(b"Hel4", 3, 2, sdk::ItemId::HelmetLv4),
    Icon::new(b"Bod1", 0, 3, sdk::ItemId::BodyArmorLv1),
    Icon::new(b"Bod2", 1, 3, sdk::ItemId::BodyArmorLv2),
    Icon::new(b"Bod3", 2, 3, sdk::ItemId::BodyArmorLv3),
    Icon::new(b"Bod4", 3, 3, sdk::ItemId::BodyArmorLv4),
    Icon::new(b"Evo1", 0, 4, sdk::ItemId::EvoShieldLv1),
    Icon::new(b"Evo2", 1, 4, sdk::ItemId::EvoShieldLv2),
    Icon::new(b"Evo3", 2, 4, sdk::ItemId::EvoShieldLv3),
    Icon::new(b"Evo4", 3, 4, sdk::ItemId::EvoShieldLv4),
    Icon::new(b"Kno1", 0, 5, sdk::ItemId::KnockdownShieldLv1),
    Icon::new(b"Kno2", 1, 5, sdk::ItemId::KnockdownShieldLv2),
    Icon::new(b"Kno3", 2, 5, sdk::ItemId::KnockdownShieldLv3),
    Icon::new(b"Kno4", 3, 5, sdk::ItemId::KnockdownShieldLv4),
    Icon::new(b"Bac1", 0, 6, sdk::ItemId::BackpackLv1),
    Icon::new(b"Bac2", 1, 6, sdk::ItemId::BackpackLv2),
    Icon::new(b"Bac3", 2, 6, sdk::ItemId::BackpackLv3),
    Icon::new(b"Bac4", 3, 6, sdk::ItemId::BackpackLv4),
    Icon::new(b"Ther", 6, 0, sdk::ItemId::Thermite),
    Icon::new(b"Frag", 7, 0, sdk::ItemId::FragGrenade),
    Icon::new(b"ArcS", 8, 0, sdk::ItemId::ArcStar),
    Icon::new(b"Clas", 0, 7, sdk::ItemId::HcogClassic),
    Icon::new(b"Brui", 1, 7, sdk::ItemId::HcogBruiser),
    Icon::new(b"Rang", 2, 7, sdk::ItemId::HcogRanger),
    Icon::new(b"Holo", 0, 8, sdk::ItemId::Holo),
    Icon::new(b"vHol", 1, 8, sdk::ItemId::VariableHolo),
    Icon::new(b"vAOG", 2, 8, sdk::ItemId::VariableAOG),
    Icon::new(b"Digi", 3, 7, sdk::ItemId::DigiThreat),
    Icon::new(b"Snip", 1, 9, sdk::ItemId::Sniper),
    Icon::new(b"vSni", 2, 9, sdk::ItemId::VariableSniper),
    Icon::new(b"DSnT", 3, 9, sdk::ItemId::DigiSniperThreat),
    Icon::new(b"BSL1", 4, 2, sdk::ItemId::BarrelStabilizerLv1),
    Icon::new(b"BSL2", 5, 2, sdk::ItemId::BarrelStabilizerLv2),
    Icon::new(b"BSL3", 6, 2, sdk::ItemId::BarrelStabilizerLv3),
    Icon::new(b"BSL4", 7, 2, sdk::ItemId::BarrelStabilizerLv4),
    Icon::new(b"BSL1", 4, 2, sdk::ItemId::LaserSightLv1),
    Icon::new(b"BSL2", 5, 2, sdk::ItemId::LaserSightLv2),
    Icon::new(b"BSL3", 6, 2, sdk::ItemId::LaserSightLv3),
    Icon::new(b"BSL4", 7, 2, sdk::ItemId::LaserSightLv4),
    Icon::new(b"LML1", 4, 3, sdk::ItemId::LightMagazineLv1),
    Icon::new(b"LML2", 5, 3, sdk::ItemId::LightMagazineLv2),
    Icon::new(b"LML3", 6, 3, sdk::ItemId::LightMagazineLv3),
    Icon::new(b"LML4", 7, 3, sdk::ItemId::LightMagazineLv4),
    Icon::new(b"HML1", 4, 4, sdk::ItemId::HeavyMagazineLv1),
    Icon::new(b"HML2", 5, 4, sdk::ItemId::HeavyMagazineLv2),
    Icon::new(b"HML3", 6, 4, sdk::ItemId::HeavyMagazineLv3),
    Icon::new(b"HML4", 7, 4, sdk::ItemId::HeavyMagazineLv4),
    Icon::new(b"EML1", 4, 5, sdk::ItemId::EnergyMagazineLv1),
    Icon::new(b"EML2", 5, 5, sdk::ItemId::EnergyMagazineLv2),
    Icon::new(b"EML3", 6, 5, sdk::ItemId::EnergyMagazineLv3),
    Icon::new(b"EML4", 7, 5, sdk::ItemId::EnergyMagazineLv4),
    Icon::new(b"SML1", 4, 6, sdk::ItemId::SniperMagazineLv1),
    Icon::new(b"SML2", 5, 6, sdk::ItemId::SniperMagazineLv2),
    Icon::new(b"SML3", 6, 6, sdk::ItemId::SniperMagazineLv3),
    Icon::new(b"SML4", 7, 6, sdk::ItemId::SniperMagazineLv4),
    Icon::new(b"SBL1", 4, 7, sdk::ItemId::ShotgunBoltLv1),
    Icon::new(b"SBL2", 5, 7, sdk::ItemId::ShotgunBoltLv2),
    Icon::new(b"SBL3", 6, 7, sdk::ItemId::ShotgunBoltLv3),
    Icon::new(b"SBL3", 6, 7, sdk::ItemId::ShotgunBoltLv4),
    Icon::new(b"StL1", 4, 8, sdk::ItemId::StandardStockLv1),
    Icon::new(b"StL2", 5, 8, sdk::ItemId::StandardStockLv2),
    Icon::new(b"StL3", 6, 8, sdk::ItemId::StandardStockLv3),
    Icon::new(b"SnL1", 4, 9, sdk::ItemId::SniperStockLv1),
    Icon::new(b"SnL2", 5, 9, sdk::ItemId::SniperStockLv2),
    Icon::new(b"SnL3", 6, 9, sdk::ItemId::SniperStockLv3),
    Icon::new(b"Hamm", 8, 6, sdk::ItemId::EpicHopUp3),
    Icon::new(b"Turb", 8, 2, sdk::ItemId::LegendaryHopUp4),
    Icon::new(b"Trea", 7, 1, sdk::ItemId::TreasurePack),
    Icon::new(b"MoRB", 6, 1, sdk::ItemId::MobileRespawn),
    Icon::new(b"MRVN", 3, 8, sdk::ItemId::MrvnArm),
];
