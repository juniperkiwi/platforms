use std::collections::BTreeMap;

use amethyst::{
    core::Transform,
    ecs::{
        prelude::*,
        world::{EntitiesRes, Index},
    },
    shred::DynamicSystemData,
};
use hibitset::BitSet;
use ncollide2d::pipeline::{object::CollisionObject, world::CollisionWorld};

use crate::collisions::{
    components::{CollisionPresence, Ncollide2dHandle},
    prelude::TransformExt,
};

pub struct Ncollide2dWorld {
    pub world: CollisionWorld<f32, Entity>,
}

impl Default for Ncollide2dWorld {
    fn default() -> Self {
        Ncollide2dWorld {
            world: CollisionWorld::new(0.02),
        }
    }
}

#[derive(Copy, Clone)]
enum ChangeType {
    None,
    Inserted,
    Modified,
    Removed,
}
impl Default for ChangeType {
    fn default() -> Self {
        ChangeType::None
    }
}
impl ChangeType {
    fn and(self, other: ChangeType) -> ChangeType {
        use ChangeType::*;
        match (self, other) {
            (None, x) => x,
            (x, None) => x,
            (Inserted, Inserted) => panic!("bad change combination"),
            (Inserted, Modified) => Inserted,
            (Inserted, Removed) => None,
            (Modified, Inserted) => panic!("bad change combiantion"),
            (Modified, Modified) => Modified,
            (Modified, Removed) => Removed,
            (Removed, Inserted) => Modified,
            (Removed, _) => panic!("bad change combination"),
        }
    }

    fn add_change(&mut self, other: ChangeType) {
        *self = self.and(other);
    }

    fn of(change: ComponentEvent) -> Self {
        match change {
            ComponentEvent::Inserted(_) => ChangeType::Inserted,
            ComponentEvent::Modified(_) => ChangeType::Modified,
            ComponentEvent::Removed(_) => ChangeType::Removed,
        }
    }
}
trait ComponentEventExt {
    fn idx(self) -> Index;
}
impl ComponentEventExt for ComponentEvent {
    fn idx(self) -> Index {
        match self {
            ComponentEvent::Inserted(idx) => idx,
            ComponentEvent::Modified(idx) => idx,
            ComponentEvent::Removed(idx) => idx,
        }
    }
}
trait UpdateCollisionObject<T> {
    fn update_from(&mut self, presence: &T);
}
impl<T> UpdateCollisionObject<CollisionPresence> for CollisionObject<f32, T> {
    fn update_from(&mut self, presence: &CollisionPresence) {
        // TODO: avoid setting all update flags (complicated by the fact that
        // none of the data structures involved implement PartialEq)
        self.set_shape(presence.shape.clone());
        self.set_collision_groups(presence.collision_groups);
        self.set_query_type(presence.query_type);
    }
}
impl<T> UpdateCollisionObject<Transform> for CollisionObject<f32, T> {
    fn update_from(&mut self, presence: &Transform) {
        self.set_position(presence.to_2d_isometry());
    }
}

#[derive(Default)]
pub struct Ncollide2dSyncPresencesSystem {
    channel: Option<ReaderId<ComponentEvent>>,
}

impl<'s> System<'s> for Ncollide2dSyncPresencesSystem {
    type SystemData = (
        Read<'s, EntitiesRes>,
        Write<'s, Ncollide2dWorld>,
        WriteStorage<'s, Ncollide2dHandle>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, CollisionPresence>,
    );

    fn run(&mut self, (entities, mut world, mut handles, transforms, presences): Self::SystemData) {
        let world = &mut world.world;

        let mut changes = BTreeMap::<Index, ChangeType>::new();

        for change in presences.channel().read(self.channel.as_mut().unwrap()) {
            let idx = change.idx();
            changes
                .entry(idx)
                .or_default()
                .add_change(ChangeType::of(*change));
        }

        let mut inserted_modified = BitSet::new();
        let mut removed = BitSet::new();
        for (idx, change) in &changes {
            match change {
                ChangeType::None => (),
                ChangeType::Inserted | ChangeType::Modified => {
                    inserted_modified.add(*idx);
                }
                ChangeType::Removed => {
                    removed.add(*idx);
                }
            }
        }

        // perform modifications
        for (presence, handle, _) in (&presences, &handles, &inserted_modified).join() {
            let collision_object = world
                .objects
                .get_mut(handle.0)
                .expect("expected CollisionWorld to have all entities with an Ncollide2dHandle");
            collision_object.update_from(presence);
        }

        // perform additions
        let mut handles_to_add = Vec::new();
        for (entity, transform, presence, _) in
            (&entities, &transforms, &presences, !&handles).join()
        {
            let (handle, _object) = world.add(
                transform.to_2d_isometry(),
                presence.shape.clone(),
                presence.collision_groups,
                presence.query_type,
                entity.clone(),
            );
            let handle = Ncollide2dHandle(handle);
            handles_to_add.push((entity, handle));
        }
        for (entity, handle) in handles_to_add {
            handles
                .insert(entity, handle)
                .expect("expected all entities in an !handles query to be missing handles");
        }
        // perform removes
        for (handle, _) in (&handles, &removed).join() {
            // ncollide2d doesn't take advantage of being passed a list, and
            // to give it a full one we'd have to collect into a vec, so let's
            // not.
            world.remove(&[handle.0]);
        }
    }

    fn setup(&mut self, world: &mut World) {
        // copied from default impl
        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world);
        // populate channel
        let mut storage = <WriteStorage<'_, CollisionPresence> as SystemData>::fetch(&world);
        self.channel.replace(storage.register_reader());
    }
}
#[derive(Default)]
pub struct Ncollide2dSyncTransformsSystem {
    channel: Option<ReaderId<ComponentEvent>>,
}

impl<'s> System<'s> for Ncollide2dSyncTransformsSystem {
    type SystemData = (
        Read<'s, EntitiesRes>,
        Write<'s, Ncollide2dWorld>,
        WriteStorage<'s, Ncollide2dHandle>,
        ReadStorage<'s, Transform>,
        ReadStorage<'s, CollisionPresence>,
    );

    fn run(&mut self, (entities, mut world, mut handles, transforms, presences): Self::SystemData) {
        let world = &mut world.world;

        let mut changes = BTreeMap::<Index, ChangeType>::new();

        for change in transforms.channel().read(self.channel.as_mut().unwrap()) {
            let idx = change.idx();
            changes
                .entry(idx)
                .or_default()
                .add_change(ChangeType::of(*change));
        }

        let mut inserted_modified = BitSet::new();
        let mut removed = BitSet::new();
        for (idx, change) in &changes {
            match change {
                ChangeType::None => (),
                ChangeType::Inserted | ChangeType::Modified => {
                    inserted_modified.add(*idx);
                }
                ChangeType::Removed => {
                    removed.add(*idx);
                }
            }
        }

        // perform modifications
        for (transform, handle, _) in (&transforms, &handles, &inserted_modified).join() {
            let collision_object = world
                .objects
                .get_mut(handle.0)
                .expect("expected CollisionWorld to have all entities with an Ncollide2dHandle");
            collision_object.update_from(transform);
        }

        // perform additions
        let mut handles_to_add = Vec::new();
        for (entity, transform, presence, _) in
            (&entities, &transforms, &presences, !&handles).join()
        {
            let (handle, _object) = world.add(
                transform.to_2d_isometry(),
                presence.shape.clone(),
                presence.collision_groups,
                presence.query_type,
                entity.clone(),
            );
            let handle = Ncollide2dHandle(handle);
            handles_to_add.push((entity, handle));
        }
        for (entity, handle) in handles_to_add {
            handles
                .insert(entity, handle)
                .expect("expected all entities in an !handles query to be missing handles");
        }
        // perform removes
        for (handle, _) in (&handles, &removed).join() {
            // ncollide2d doesn't take advantage of being passed a list, and
            // to give it a full one we'd have to collect into a vec, so let's
            // not.
            world.remove(&[handle.0]);
        }
    }

    fn setup(&mut self, world: &mut World) {
        // copied from default impl
        <Self::SystemData as DynamicSystemData>::setup(&self.accessor(), world);
        // populate channel
        let mut storage = <WriteStorage<'_, Transform> as SystemData>::fetch(&world);
        self.channel.replace(storage.register_reader());
    }
}

#[derive(Default)]
pub struct Ncollide2dUpdateWorldSystem;

impl<'s> System<'s> for Ncollide2dUpdateWorldSystem {
    type SystemData = (Write<'s, Ncollide2dWorld>,);

    fn run(&mut self, (mut ncollide_world,): Self::SystemData) {
        let world = &mut ncollide_world.world;
        world.update();
        // eprintln!(
        //     "{:#?}",
        //     world
        //         .objects
        //         .iter()
        //         .map(|v| format!("{:?}", (v.1.collision_groups(), v.1.position())))
        //         .collect::<Vec<_>>()
        // );
    }
}
