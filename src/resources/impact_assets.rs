use bevy::{asset::Handle, mesh::Mesh, pbr::StandardMaterial, prelude::Resource};

#[derive(Resource)]
pub struct ImpactAssets {
    pub radius: f32,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub lifetime_secs: f32,
}
