use crate::physics::collider::Collider;
use bevy::prelude::*;

#[derive(Component)]
pub struct Rigidbody {
    pub velocity: Vec2,
    pub angular_velocity: f32,
    pub kinematic: bool,
}

#[derive(Bundle)]
pub struct RigidbodyBundle {
    pub rb: Rigidbody,
    pub col: Collider,
}

impl Default for Rigidbody {
    fn default() -> Self {
        Self {
            kinematic: false,
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
        }
    }
}

impl Rigidbody {
    pub fn zero(kinematic: bool) -> Self {
        Self {
            kinematic,
            ..Default::default()
        }
    }
}

fn movement(mut query: Query<(&mut Transform, &Rigidbody)>) {
    for (mut trans, rb) in query.iter_mut() {
        trans.translation.x += rb.velocity.x;
        trans.translation.y += rb.velocity.y;
    }
}

pub struct RigidbodyPlugin;

impl Plugin for RigidbodyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement.label("movement"));
    }
}
