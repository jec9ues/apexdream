use crate::base::solver::{Collection, Trajectory};

pub use self::bocek::COLLECTION as BOCEK;
pub use self::devotion::COLLECTION as DEVOTION;
pub use self::flatline::COLLECTION as FLATLINE;
pub use self::g7_scout::COLLECTION as G7_SCOUT;
pub use self::hemlok::COLLECTION as HEMLOK;
pub use self::kraber::COLLECTION as KRABER;
pub use self::longbow::COLLECTION as LONGBOW;
pub use self::prowler::COLLECTION as PROWLER;
pub use self::r301::COLLECTION as R301;
pub use self::repeater::COLLECTION as REPEATER;
pub use self::sentinel::COLLECTION as SENTINEL;
pub use self::spitfire::COLLECTION as SPITFIRE;
pub use self::throwing_knife::COLLECTION as THROWING_KNIFE;
pub use self::volt::COLLECTION as VOLT;
pub use self::wingman::COLLECTION as WINGMAN;

mod sentinel;
mod kraber;
mod bocek;
mod r301;
mod g7_scout;
mod repeater;
mod longbow;
mod flatline;
mod throwing_knife;
mod wingman;
mod prowler;
mod volt;
mod devotion;
mod spitfire;
mod hemlok;
