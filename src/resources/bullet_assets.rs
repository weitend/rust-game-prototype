use bevy::{asset::Handle, mesh::Mesh, pbr::StandardMaterial, prelude::Resource};

#[derive(Resource)]
pub struct BulletAssets {
    pub radius: f32,
    pub speed: f32,
    pub bullet_lifetime_secs: f32,

    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,

    pub impact_radius: f32,
    pub impact_mesh: Handle<Mesh>,
    pub impact_material: Handle<StandardMaterial>,
}
