pub mod collider;
pub mod rigidbody;
pub mod trigger;

use crate::App;
use bevy::prelude::Plugin;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(collider::ColliderPlugin);
        app.add_plugin(rigidbody::RigidbodyPlugin);
    }
}
