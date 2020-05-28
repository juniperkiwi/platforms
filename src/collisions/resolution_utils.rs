use nalgebra::{Isometry2, Unit, Vector2};
use ncollide2d::{
    pipeline::{object::CollisionObjectSlabHandle, CollisionObject, CollisionWorld},
    query::Contact,
    shape::Shape,
};

use super::prelude::IsometryExt;

const DIRECTION_TEST_DELTA: f32 = 0.001;

pub fn contact_in_direction<T>(
    obj1: &CollisionObject<f32, T>,
    obj2: &CollisionObject<f32, T>,
    direction: Unit<Vector2<f32>>,
) -> Option<Contact<f32>> {
    let isometry = obj1.position();
    let in_depth = isometry.prepend_movement(direction, DIRECTION_TEST_DELTA);
    let contact1 = ncollide2d::query::contact(
        obj1.position(),
        &**obj1.shape(),
        obj2.position(),
        &**obj2.shape(),
        0.0,
    )?;
    let contact2 = ncollide2d::query::contact(
        &in_depth,
        &**obj1.shape(),
        obj2.position(),
        &**obj2.shape(),
        0.0,
    )?;
    if contact2.depth > contact1.depth {
        Some(contact2)
    } else {
        None
    }
}

pub fn contact_in_direction_with_shape<T>(
    isometry: &Isometry2<f32>,
    shape: &dyn Shape<f32>,
    obj2: &CollisionObject<f32, T>,
    direction: Unit<Vector2<f32>>,
) -> Option<Contact<f32>> {
    let in_depth = isometry.prepend_movement(direction, DIRECTION_TEST_DELTA);
    let contact1 =
        ncollide2d::query::contact(isometry, shape, obj2.position(), &**obj2.shape(), 0.0)?;
    let contact2 =
        ncollide2d::query::contact(&in_depth, shape, obj2.position(), &**obj2.shape(), 0.0)?;
    if contact2.depth > contact1.depth {
        Some(contact2)
    } else {
        None
    }
}

pub fn on_floor<T>(
    ncollide_world: &CollisionWorld<f32, T>,
    handle: CollisionObjectSlabHandle,
) -> bool {
    ncollide_world
        .contacts_with(handle, true)
        .into_iter()
        .flat_map(|v| v)
        .any(|(handle1, handle2, _algo, manifold)| {
            contact_in_direction(
                ncollide_world.objects.get(handle1).unwrap(),
                ncollide_world.objects.get(handle2).unwrap(),
                -Vector2::y_axis(),
            )
            .is_some()
        })
}
