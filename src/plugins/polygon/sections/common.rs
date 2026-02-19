use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        combat::{Health, Team},
        obstacle::Obstacle,
    },
    plugins::polygon::{config::PolygonConfig, layout::SectionBounds},
    utils::collision_groups::{GROUP_ENEMY, GROUP_PLAYER, GROUP_WORLD},
};

pub fn spawn_platform(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    config: &PolygonConfig,
) {
    let size = config.platform_size();

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, -0.5 * size.y, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(0.5 * size.x, 0.5 * size.y, 0.5 * size.z),
        CollisionGroups::new(GROUP_WORLD, Group::ALL),
        Friction::coefficient(0.0),
    ));
}

pub fn spawn_static_block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
    mark_obstacle: bool,
) {
    let mut entity = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(position),
        RigidBody::Fixed,
        Collider::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
        CollisionGroups::new(GROUP_WORLD, Group::ALL),
    ));

    if mark_obstacle {
        entity.insert((Obstacle, ActiveEvents::COLLISION_EVENTS));
    }
}

pub fn spawn_visual_block(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(position),
    ));
}

pub fn spawn_damage_dummy(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<StandardMaterial>,
    position: Vec3,
    size: Vec3,
    team: Team,
    max_health: f32,
) {
    let group = match team {
        Team::Player => GROUP_PLAYER,
        Team::Enemy => GROUP_ENEMY,
    };

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(position),
        RigidBody::Fixed,
        Collider::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
        CollisionGroups::new(group, Group::ALL),
        team,
        Health::new(max_health),
    ));
}

pub fn section_center(config: &PolygonConfig, bounds: SectionBounds) -> Vec3 {
    let min = config.module_center(bounds.min_col, bounds.min_row);
    let max = config.module_center(bounds.max_col, bounds.max_row);

    Vec3::new((min.x + max.x) * 0.5, 0.0, (min.z + max.z) * 0.5)
}

pub fn section_span(config: &PolygonConfig, bounds: SectionBounds) -> Vec2 {
    Vec2::new(
        config.span_for_modules(bounds.width_modules()),
        config.span_for_modules(bounds.height_modules()),
    )
}
