#![feature(int_roundings)]

mod debug;
mod flowfield;
mod physics;
mod player;
mod zombie;

use crate::physics::collider::Collider;
use crate::physics::rigidbody::{Rigidbody, RigidbodyBundle};
use bevy::prelude::*;
use bevy::render::settings::{Backends, WgpuSettings};
use ncollide2d::na::{Isometry2, Vector2};
use ncollide2d::shape::Cuboid;
use rand::Rng;

const WINDOW_WIDTH: usize = 1280;
const WINDOW_HEIGHT: usize = 720;

#[derive(Component)]
pub struct Wall;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle({
        let mut camera = OrthographicCameraBundle::new_2d();
        camera.transform.translation.x += WINDOW_WIDTH as f32 / 2.;
        camera.transform.translation.y += WINDOW_HEIGHT as f32 / 2.;
        camera
    });

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::Rgba {
                    red: 0.3,
                    green: 0.3,
                    blue: 0.8,
                    alpha: 0.5,
                },
                custom_size: Some(Vec2::new(10., 200.)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(200., 200., 0.),
                rotation: Quat::from_rotation_z(0.),
                ..Default::default()
            },
            texture: asset_server.load("square.png"),
            ..Default::default()
        })
        .insert_bundle(RigidbodyBundle {
            rb: Rigidbody::zero(true),
            col: Collider::new(
                Box::new(Cuboid {
                    half_extents: Vector2::new(5., 100.),
                }),
                0,
            ),
        })
        .insert(Wall);
}

fn main() {
    App::new()
        .insert_resource(WgpuSettings {
            backends: Some(Backends::DX12),
            ..Default::default()
        })
        .insert_resource(WindowDescriptor {
            width: WINDOW_WIDTH as f32,
            height: WINDOW_HEIGHT as f32,
            title: "My Game".to_string(),
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(debug::DebugPlugin)
        .add_startup_system(setup)
        .add_plugin(physics::PhysicsPlugin)
        .add_plugin(flowfield::FlowFieldPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(zombie::ZombiePlugin)
        .run();
}
