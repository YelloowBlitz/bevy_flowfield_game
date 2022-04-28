use bevy::prelude::*;
use crate::{Collider, Rigidbody};
use crate::flowfield::Flowing;
use crate::physics::trigger::Trigger;

#[derive(Component)]
pub struct Bullet;

#[derive(Bundle)]
pub struct BulletBundle {
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    pub col: Collider,
    pub trig: Trigger,
    pub rb: Rigidbody,
    pub _bullet: Bullet,
}

fn bullet_trigger(
    query_bullet: Query<(Entity, &Transform, &Collider), (With<Trigger>, With<Bullet>)>,
    query_others: Query<(Entity, &Transform, &Collider), (Without<Bullet>, With<Flowing>)>,
    mut commands: Commands,
) {
    for (e1, trans1, col1) in query_bullet.iter() {
        for (e2, trans2, col2) in query_others.iter() {
            if col1.overlap(trans1, col2, trans2) {
                commands.entity(e1).despawn();
                commands.entity(e2).despawn();
            }
        }
    }
}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(bullet_trigger);
    }
}
