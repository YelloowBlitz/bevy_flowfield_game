use crate::physics::rigidbody::Rigidbody;
use crate::physics::trigger::Trigger;
use bevy::prelude::*;
use ncollide2d::na::{Isometry2, Point2, Unit, Vector2};
use ncollide2d::query::{proximity, Proximity, Ray, RayCast};
use ncollide2d::shape::{Cuboid, Shape};
use std::any::Any;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};

pub type Layer = u32;

#[derive(Component)]
pub struct Collider {
    shape: Box<dyn Shape<f32>>,
    pub layer: Layer,
}

impl Debug for Collider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(c) = self.shape.as_any().downcast_ref::<Cuboid<f32>>() {
            write!(f, "Cuboid {:?}", c.half_extents)
        } else {
            write!(f, "UNKNWON SHAPE")
        }
    }
}

impl Collider {
    pub fn new(shape: Box<dyn Shape<f32>>, layer: Layer) -> Self {
        Self { shape, layer }
    }

    pub fn overlap(&self, self_trans: &Transform, other: &Self, other_trans: &Transform) -> bool {
        let test = proximity(
            &Isometry2::new(
                Vector2::new(self_trans.translation.x, self_trans.translation.y),
                self_trans.rotation.angle_between(Quat::from_rotation_z(0.)),
            ),
            &*self.shape,
            &Isometry2::new(
                Vector2::new(other_trans.translation.x, other_trans.translation.y),
                other_trans
                    .rotation
                    .angle_between(Quat::from_rotation_z(0.)),
            ),
            &*other.shape,
            0.,
        );

        test == Proximity::Intersecting
    }

    pub fn raycast(&self, self_trans: &Transform, ray: &Ray<f32>, max_dist: f32) -> Option<f32> {
        let iso = Isometry2::new(Vector2::new(self_trans.translation.x, self_trans.translation.y), self_trans.rotation.angle_between(Quat::from_rotation_z(0.)));

        self.shape.toi_with_ray(&iso, ray, max_dist, true)
    }
}

pub type LayersInteraction = HashMap<Layer, HashSet<Layer>>;

fn setup_layers(mut commands: Commands) {
    let mut layer_map: LayersInteraction = HashMap::new();

    layer_map.insert(0, HashSet::new());
    layer_map.get_mut(&0).unwrap().insert(0);

    commands.insert_resource(layer_map)
}

fn collisions(
    mut query: Query<(&mut Transform, &Collider, &Rigidbody), Without<Trigger>>,
    layers: Res<LayersInteraction>,
) {
    let mut iter = query.iter_combinations_mut::<2>();
    while let Some([(mut trans1, col1, rb1), (mut trans2, col2, rb2)]) = iter.fetch_next() {
        if layers
            .get(&col1.layer)
            .unwrap_or(&HashSet::new())
            .contains(&col2.layer)
            || layers
                .get(&col2.layer)
                .unwrap_or(&HashSet::new())
                .contains(&col1.layer)
        {
            let iso1 = Isometry2::new(
                Vector2::new(trans1.translation.x, trans1.translation.y),
                trans1.rotation.angle_between(Quat::from_rotation_z(0.)),
            );

            let iso2 = Isometry2::new(
                Vector2::new(trans2.translation.x, trans2.translation.y),
                trans2.rotation.angle_between(Quat::from_rotation_z(0.)),
            );

            if let Some(contact) = ncollide2d::query::contact(
                &iso1,
                col1.shape.borrow(),
                &iso2,
                col2.shape.borrow(),
                0.0,
            ) {
                let mov_dir: Unit<Vector2<f32>> = contact.normal;
                if rb2.kinematic {
                    trans1.translation.x -= mov_dir.x * contact.depth;
                    trans1.translation.y -= mov_dir.y * contact.depth;
                } else if rb1.kinematic {
                    trans2.translation.x += mov_dir.x * contact.depth;
                    trans2.translation.y += mov_dir.y * contact.depth;
                }
            }
        }
    }
}

pub struct ColliderPlugin;

impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_layers)
            .add_system(collisions.after("movement"));
    }
}
