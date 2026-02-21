use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        combat::{Health, Team},
        destructible_mesh::DestructibleMesh,
        destructible_surface::DestructibleSurface,
        obstacle::Obstacle,
    },
    plugins::polygon::{config::PolygonConfig, layout::SectionBounds},
    utils::collision_groups::{GROUP_ENEMY, GROUP_PLAYER, GROUP_WORLD},
};

const DESTRUCTIBLE_VERTEX_SPACING: f32 = 0.14;
const DESTRUCTIBLE_MIN_FACE_STEPS: u32 = 8;
const DESTRUCTIBLE_MAX_FACE_STEPS: u32 = 96;

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
    let mesh = if mark_obstacle {
        meshes.add(build_destructible_block_mesh(size))
    } else {
        meshes.add(Cuboid::new(size.x, size.y, size.z))
    };

    let mut entity = commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(position),
        RigidBody::Fixed,
        Collider::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
        CollisionGroups::new(GROUP_WORLD, Group::ALL),
    ));

    if mark_obstacle {
        entity.insert((
            Obstacle,
            DestructibleSurface::default(),
            DestructibleMesh::for_size(size),
        ));
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

fn build_destructible_block_mesh(size: Vec3) -> Mesh {
    let mut positions = Vec::<[f32; 3]>::new();
    let mut normals = Vec::<[f32; 3]>::new();
    let mut uvs = Vec::<[f32; 2]>::new();
    let mut indices = Vec::<u32>::new();

    let hx = size.x * 0.5;
    let hy = size.y * 0.5;
    let hz = size.z * 0.5;

    push_grid_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(0.0, 0.0, hz),
        Vec3::new(size.x, 0.0, 0.0),
        Vec3::new(0.0, size.y, 0.0),
        Vec3::Z,
        face_steps(size.x),
        face_steps(size.y),
    );
    push_grid_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(0.0, 0.0, -hz),
        Vec3::new(-size.x, 0.0, 0.0),
        Vec3::new(0.0, size.y, 0.0),
        -Vec3::Z,
        face_steps(size.x),
        face_steps(size.y),
    );
    push_grid_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(hx, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -size.z),
        Vec3::new(0.0, size.y, 0.0),
        Vec3::X,
        face_steps(size.z),
        face_steps(size.y),
    );
    push_grid_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(-hx, 0.0, 0.0),
        Vec3::new(0.0, 0.0, size.z),
        Vec3::new(0.0, size.y, 0.0),
        -Vec3::X,
        face_steps(size.z),
        face_steps(size.y),
    );
    push_grid_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(0.0, hy, 0.0),
        Vec3::new(size.x, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -size.z),
        Vec3::Y,
        face_steps(size.x),
        face_steps(size.z),
    );
    push_grid_face(
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut indices,
        Vec3::new(0.0, -hy, 0.0),
        Vec3::new(size.x, 0.0, 0.0),
        Vec3::new(0.0, 0.0, size.z),
        -Vec3::Y,
        face_steps(size.x),
        face_steps(size.z),
    );

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn face_steps(axis_len: f32) -> u32 {
    let safe = axis_len.max(0.01);
    let raw = (safe / DESTRUCTIBLE_VERTEX_SPACING).ceil() as u32;
    raw.clamp(DESTRUCTIBLE_MIN_FACE_STEPS, DESTRUCTIBLE_MAX_FACE_STEPS)
}

fn push_grid_face(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec3,
    u_axis: Vec3,
    v_axis: Vec3,
    normal: Vec3,
    subdivisions_u: u32,
    subdivisions_v: u32,
) {
    let steps_u = subdivisions_u.max(1);
    let steps_v = subdivisions_v.max(1);
    let row_width = steps_u + 1;
    let base = positions.len() as u32;

    for y in 0..=steps_v {
        let v = y as f32 / steps_v as f32;
        for x in 0..=steps_u {
            let u = x as f32 / steps_u as f32;
            let position = center + u_axis * (u - 0.5) + v_axis * (v - 0.5);
            positions.push(position.to_array());
            normals.push(normal.to_array());
            uvs.push([u, v]);
        }
    }

    for y in 0..steps_v {
        for x in 0..steps_u {
            let i0 = base + y * row_width + x;
            let i1 = i0 + 1;
            let i2 = i0 + row_width;
            let i3 = i2 + 1;

            indices.push(i0);
            indices.push(i1);
            indices.push(i2);
            indices.push(i1);
            indices.push(i3);
            indices.push(i2);
        }
    }
}
