use amethyst::{
    core::{timing::Time, SystemDesc, Transform},
    derive::SystemDesc,
    ecs::prelude::*,
    input::{InputHandler, StringBindings},
};
use ncollide2d::{
    bounding_volume::bounding_volume::BoundingVolume,
    pipeline::{
        narrow_phase::ContactAlgorithm, object::CollisionObjectSlabHandle, CollisionGroups,
        CollisionObject, CollisionWorld, GeometricQueryType,
    },
    query::{Contact, ContactManifold},
    shape::{Shape, ShapeHandle},
};
use specs_derive::Component;

#[derive(Clone)]
pub struct CollisionPresence {
    pub shape: ShapeHandle<f32>,
    pub collision_groups: CollisionGroups,
    pub query_type: GeometricQueryType<f32>,
}

impl Component for CollisionPresence {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct Ncollide2dHandle(pub(crate) CollisionObjectSlabHandle);

#[derive(Default, Debug, Component)]
#[storage(NullStorage)]
pub struct HasGravity;
