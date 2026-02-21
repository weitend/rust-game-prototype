use bevy::{asset::Handle, mesh::Mesh, pbr::StandardMaterial, prelude::Resource};

#[derive(Resource)]
pub struct ImpactAssets {
    pub radius: f32,
    pub crater_size: f32,
    pub crater_depth: f32,
    pub min_marks_per_impact: usize,
    pub max_marks_per_impact: usize,
    pub damage_for_max_web: f32,
    pub base_web_radius: f32,
    pub max_web_radius: f32,
    pub max_marks_per_frame: usize,
    pub chip_mesh: Handle<Mesh>,
    pub chip_fallback_material: Handle<StandardMaterial>,
    pub min_chips_per_impact: usize,
    pub max_chips_per_impact: usize,
    pub chip_size: f32,
    pub chip_speed: f32,
    pub chip_lifetime_secs: f32,
    pub max_chips_per_frame: usize,
}
