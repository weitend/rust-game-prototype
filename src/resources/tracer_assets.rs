use bevy::{asset::Handle, mesh::Mesh, pbr::StandardMaterial, prelude::Resource};

#[derive(Resource)]
pub struct TracerAssets {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub speed: f32,
    pub smoke_mesh: Handle<Mesh>,
    pub smoke_material: Handle<StandardMaterial>,
    pub explosion_mesh: Handle<Mesh>,
    pub explosion_material: Handle<StandardMaterial>,
}
