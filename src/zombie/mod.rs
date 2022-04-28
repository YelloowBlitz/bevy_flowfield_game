use bevy::prelude::*;
use ncollide2d::na::{Point2, Vector2};
use ncollide2d::query::RayCast;
use ncollide2d::shape::Cuboid;
use crate::{Collider, Rigidbody, RigidbodyBundle, Wall};
use crate::flowfield::Flowing;
use crate::physics::trigger::Trigger;
use crate::player::Player;

#[derive(Component)]
pub struct Zombie;

#[derive(Bundle)]
pub struct ZombieBundle {
    #[bundle]
    pub rigidbody: RigidbodyBundle,
    #[bundle]
    pub sprite: SpriteBundle,
    pub _f: Flowing,
    _z: Zombie,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let zombie = ZombieBundle {
        rigidbody: RigidbodyBundle {
            rb: Rigidbody::zero(false),
            col: Collider::new(Box::new(Cuboid::new(Vector2::new(10., 10.))), 0)
        },
        sprite: SpriteBundle {
            sprite: Sprite {
                color: Color::Rgba {
                    red: 1.0,
                    green: 0.0,
                    blue: 0.0,
                    alpha: 1.0
                },
                custom_size: Some(Vec2::new(10., 10.)),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(50., 50., 0.),
                ..Default::default()
            },
            texture: asset_server.load("square.png"),
            ..Default::default()
        },
        _f: Flowing,
        _z: Zombie
    };

    commands.spawn_bundle(zombie);
}

fn zombie_movement(
    mut zombie_query: Query<(&mut Rigidbody, &Transform), With<Zombie>>,
    player_query: Query<&Transform, With<Player>>,
    collider_query: Query<(&Collider, &Transform), (Without<Trigger>, With<Wall>)>
) {
    for (mut zombie_rb, zombie_trans) in zombie_query.iter_mut() {
        for player_trans in player_query.iter() {
            // TODO check layer too and improve all this
            let player_pos = ncollide2d::na::Vector2::new(player_trans.translation.x, player_trans.translation.y);
            if collider_query.iter().all(|(col, col_trans)| {
                let zombie_pos = ncollide2d::na::Vector2::new(zombie_trans.translation.x, zombie_trans.translation.y);

                let max_dist = ncollide2d::na::distance(&ncollide2d::na::Point2::from(player_pos), &ncollide2d::na::Point2::from(zombie_pos));
                // println!("start : {} | end : {} | length : {}", zombie_pos, player_pos, max_dist);
                // let a: Vector2<f32> = Vector2::new(0., 0.);
                let dir = player_pos - zombie_pos;
                if let Some(dist) = col.raycast(
                    &col_trans,
                    &ncollide2d::query::Ray::new(
                        ncollide2d::na::Point2::from(zombie_pos),
                        dir / dir.norm()),
                    max_dist
                ) {
                    println!("toi : {} | max_dist : {}", dist, max_dist);
                    false
                }
                else {
                    true
                }
            }) {
                zombie_rb.velocity.x = player_pos.x - zombie_trans.translation.x;
                zombie_rb.velocity.y = player_pos.y - zombie_trans.translation.y;
                zombie_rb.velocity = zombie_rb.velocity.normalize_or_zero();

                println!("Nothing in sight !");
            } else {
                println!("Something in sight !");
            }
        }
    }
}

pub struct ZombiePlugin;

impl Plugin for ZombiePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, setup);
        app.add_system(zombie_movement.after("flowing").before("movement"));
    }
}