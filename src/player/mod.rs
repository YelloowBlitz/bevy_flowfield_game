mod bullet;

use crate::physics;
use crate::physics::collider::Collider;
use crate::physics::rigidbody::{Rigidbody, RigidbodyBundle};
use bevy::prelude::*;
use ncollide2d::na::Vector2;
use ncollide2d::shape::Cuboid;
use std::collections::HashSet;
use crate::physics::trigger::Trigger;
use crate::player::bullet::{Bullet, BulletBundle};

type GamepadUsed = HashSet<Gamepad>;

#[derive(Component)]
pub enum Controllable {
    GAMEPAD(Gamepad),
    KEYBOARD,
}

impl Controllable {
    pub fn get_dir(&self, axes: &Res<Axis<GamepadAxis>>, keys: &Res<Input<KeyCode>>) -> Option<Vec2> {
        match self {
            Controllable::GAMEPAD(g) => {
                let axis_lx = GamepadAxis(*g, GamepadAxisType::LeftStickX);
                let axis_ly = GamepadAxis(*g, GamepadAxisType::LeftStickY);

                if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
                    Some(Vec2::new(x, y).normalize_or_zero())
                } else {
                    None
                }
            }
            Controllable::KEYBOARD => {
                let mut dir = Vec2::ZERO;
                if keys.pressed(KeyCode::Right) {
                    dir.x += 1.;
                }
                if keys.pressed(KeyCode::Left) {
                    dir.x -= 1.;
                }
                if keys.pressed(KeyCode::Up) {
                    dir.y += 1.;
                }
                if keys.pressed(KeyCode::Down) {
                    dir.y -= 1.;
                }

                Some(dir.normalize_or_zero())
            }
        }
    }
}

#[derive(Component)]
pub struct Player {
    last_dir: Vec2,
    fire_cooldown: f64,
    fire_timing: f64,
}

impl Default for Player {
    fn default() -> Self {
        Self { last_dir: Vec2::new(0., 1.), fire_cooldown: 0.2, fire_timing: 0.0 }
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub controller: Controllable,
    #[bundle]
    pub rigidbody: RigidbodyBundle,
    #[bundle]
    pub sprite: SpriteBundle,
    _p: Player,
}

fn new_gamepad(
    mut commands: Commands,
    mut gamepad_evr: EventReader<GamepadEvent>,
    keys: Res<Input<KeyCode>>,
    mut gamepads_used: ResMut<GamepadUsed>,
    asset_server: Res<AssetServer>,
) {
    for GamepadEvent(gamepad, event_type) in gamepad_evr.iter() {
        if !gamepads_used.contains(&gamepad) {
            if let GamepadEventType::ButtonChanged(GamepadButtonType::South, val) = event_type {
                if *val > 0.5 {
                    println!("Spawn new player !");
                    commands.spawn_bundle(PlayerBundle {
                        controller: Controllable::GAMEPAD(*gamepad),
                        rigidbody: RigidbodyBundle {
                            rb: Rigidbody::zero(false),
                            col: Collider::new(Box::new(Cuboid::new(Vector2::new(5., 5.))), 0),
                        },
                        sprite: SpriteBundle {
                            sprite: Sprite {
                                color: Color::Rgba {
                                    red: 0.0,
                                    green: 0.0,
                                    blue: 0.0,
                                    alpha: 1.0,
                                },
                                custom_size: Some(Vec2::new(10.0, 10.0)),
                                ..Default::default()
                            },
                            transform: Transform {
                                translation: Vec3::new(100., 100., 0.),
                                ..Default::default()
                            },
                            texture: asset_server.load("player.png"),
                            ..Default::default()
                        },
                        _p: Player::default(),
                    });

                    gamepads_used.insert(*gamepad);
                }
            }
        }
    }

    if keys.just_pressed(KeyCode::P) && !gamepads_used.contains(&Gamepad(usize::MAX)) {
        println!("Spawn new player !");
        commands
            .spawn_bundle(PlayerBundle {
                controller: Controllable::KEYBOARD,
                rigidbody: RigidbodyBundle {
                    rb: Rigidbody::zero(false),
                    col: Collider::new(Box::new(Cuboid::new(Vector2::new(5., 5.))), 0),
                },
                sprite: SpriteBundle {
                    sprite: Sprite {
                        color: Color::Rgba {
                            red: 0.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha: 1.0,
                        },
                        custom_size: Some(Vec2::new(10.0, 10.0)),
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: Vec3::new(100., 100., 0.),
                        ..Default::default()
                    },
                    texture: asset_server.load("player.png"),
                    ..Default::default()
                },
                _p: Player::default(),
            })
            .insert_bundle(RigidbodyBundle {
                rb: Rigidbody::zero(false),
                col: Collider::new(Box::new(Cuboid::new(Vector2::new(5., 5.))), 0),
            });
        // .insert(Collider::new(Box::new(Cuboid::new(Vector2::new(5., 5.))), 0, false));
        // TODO Maybe to something here
        gamepads_used.insert(Gamepad(usize::MAX));
    }
}

fn player_movement(
    mut players: Query<(&mut Rigidbody, &Controllable, &mut Player)>,
    axes: Res<Axis<GamepadAxis>>,
    keys: Res<Input<KeyCode>>,
) {
    for (mut rb, c, mut p) in players.iter_mut() {
        let dir_opt = c.get_dir(&axes, &keys);

        if let Some(dir) = dir_opt {
            rb.velocity.x = dir.x * 3.;
            rb.velocity.y = dir.y * 3.;
            if dir != Vec2::ZERO {
                p.last_dir = dir;
            }
        } else {
            rb.velocity = Vec2::ZERO;
        }
    }
}

fn fire_bullet(
    mut players: Query<(&Transform, &Controllable, &mut Player)>,
    buttons: Res<Input<GamepadButton>>,
    axes: Res<Axis<GamepadAxis>>,
    keys: Res<Input<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    for (trans, c, mut p) in players.iter_mut() {
        if match c {
            Controllable::GAMEPAD(g) => {
                let fire_button = GamepadButton(*g, GamepadButtonType::West);

                buttons.pressed(fire_button)
            }
            Controllable::KEYBOARD => {
                keys.pressed(KeyCode::F)
            }
        } {
            let time_startup = time.seconds_since_startup();
            if p.fire_cooldown + p.fire_timing < time_startup {
                p.fire_timing = time_startup;
                let dir = c.get_dir(&axes, &keys).filter(|d| *d != Vec2::ZERO).unwrap_or(p.last_dir);
                commands.spawn_bundle(BulletBundle {
                    sprite_bundle: SpriteBundle {
                        sprite: Sprite {
                            color: Color::Rgba {
                                red: 1.0,
                                green: 0.0,
                                blue: 0.0,
                                alpha: 1.0
                            },
                            custom_size: Some(Vec2::new(4., 4.)),
                            ..Default::default()
                        },
                        transform: Transform {
                            translation: Vec3::new(trans.translation.x, trans.translation.y, 0.),
                            ..Default::default()
                        },
                        texture: asset_server.load("square.png"),
                        ..Default::default()
                    },
                    col: Collider::new(
                        Box::new(Cuboid::new(Vector2::new(2., 2.))),
                        0),
                    trig: Trigger,
                    rb: Rigidbody {
                        velocity: 3. * dir,
                        angular_velocity: 0.0,
                        kinematic: false
                    },
                    _bullet: Bullet
                });
            }
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GamepadUsed::new());
        app.add_plugin(bullet::BulletPlugin);
        app.add_system(new_gamepad);
        app.add_system(player_movement);
        app.add_system(fire_bullet);
    }
}
