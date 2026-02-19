use bevy::{asset::Handle, mesh::Mesh, pbr::StandardMaterial, prelude::Resource};

#[derive(Resource)]
pub struct TracerAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub speed: f32,
}
