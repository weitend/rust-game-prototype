use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    components::{
        bullet::{Bullet, BulletLifeTime},
        fire_control::FireControl,
        player::Player,
        shoot_origin::ShootOrigin,
    },
    resources::bullet_assets::BulletAssets,
    utils::collision_groups::bullet_collision_groups,
};

pub fn fire_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&Transform, &ShootOrigin, &mut FireControl), With<Player>>,
    bullet_assets: Res<BulletAssets>,
    time: Res<Time>,
) {
    if mouse.pressed(MouseButton::Left) {
        let (player_tf, shoot_origin, mut fire_control) =
            query.single_mut().expect("Failed to get Player query");

        fire_control.cooldown.tick(time.delta());

        if mouse.just_pressed(MouseButton::Left) || fire_control.cooldown.just_finished() {
            let forward = player_tf.rotation * -Vec3::Z;

            commands.spawn((
                Mesh3d(bullet_assets.mesh.clone()),
                MeshMaterial3d(bullet_assets.material.clone()),
                Transform::from_translation(
                    player_tf.translation + player_tf.rotation * shoot_origin.muzzle_offset,
                ),
                Bullet,
                RigidBody::Dynamic,
                Collider::ball(bullet_assets.radius),
                bullet_collision_groups(),
                GravityScale(0.0),
                Ccd::enabled(),
                Velocity::linear(forward * bullet_assets.speed),
                BulletLifeTime {
                    timer: Timer::from_seconds(bullet_assets.bullet_lifetime_secs, TimerMode::Once),
                },
                ActiveEvents::COLLISION_EVENTS,
            ));
        }
    } else {
        let (_, _, mut fire_control) = query
            .single_mut()
            .expect("Failed to get Player fire_control");

        fire_control.cooldown.reset();
    };
}
