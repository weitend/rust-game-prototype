mod damage;
mod visual;

pub use damage::{ImpactEvent, route_impact_damage_system};
pub use visual::{debris_chip_lifetime_system, process_impact_system};
