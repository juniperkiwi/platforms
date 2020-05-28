use amethyst::ecs::prelude::*;
use ncollide2d::{
    pipeline::{object::CollisionObjectSlabHandle, CollisionGroups, GeometricQueryType},
    shape::ShapeHandle,
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
