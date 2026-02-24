mod damage;
mod visual;

pub use damage::{route_impact_damage_system, ImpactEvent};
pub use visual::{debris_chip_lifetime_system, process_impact_system};
