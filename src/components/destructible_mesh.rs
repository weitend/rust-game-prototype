use bevy::{math::Vec3, prelude::Component};

#[derive(Component, Clone, Copy, Debug)]
pub struct DestructibleMesh {
    pub max_dent_depth: f32,
}

impl DestructibleMesh {
    pub fn for_size(size: Vec3) -> Self {
        let min_extent = size.min_element().max(0.1);
        Self {
            max_dent_depth: min_extent * 0.42,
        }
    }
}
