use crate::{
    archetype::ArchetypeComponentId,
    storage::SparseSet,
    system::Resource,
    world::{Mut, World},
};
use std::{
    any::TypeId,
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

/// Exposes safe mutable access to multiple resources at a time in a World. Attempting to access
/// World in a way that violates Rust's mutability rules will panic thanks to runtime checks.
pub struct WorldCell<'w> {
    pub(crate) world: &'w mut World,
    pub(crate) access: Rc<RefCell<ArchetypeComponentAccess>>,
}

pub(crate) struct ArchetypeComponentAccess {
    access: SparseSet<ArchetypeComponentId, usize>,
}

impl Default for ArchetypeComponentAccess {
    fn default() -> Self {
        Self {
            access: SparseSet::new(),
        }
    }
}

const UNIQUE_ACCESS: usize = 0;
const BASE_ACCESS: usize = 1;
impl ArchetypeComponentAccess {
    const fn new() -> Self {
        Self {
            access: SparseSet::new(),
        }
    }

    fn read(&mut self, id: ArchetypeComponentId) -> bool {
        let id_access = self.access.get_or_insert_with(id, || BASE_ACCESS);
        if *id_access == UNIQUE_ACCESS {
            false
        } else {
            *id_access += 1;
            true
        }
    }

    fn drop_read(&mut self, id: ArchetypeComponentId) {
        let id_access = self.access.get_or_insert_with(id, || BASE_ACCESS);
        *id_access -= 1;
    }

    fn write(&mut self, id: ArchetypeComponentId) -> bool {
        let id_access = self.access.get_or_insert_with(id, || BASE_ACCESS);
        if *id_access == BASE_ACCESS {
            *id_access = UNIQUE_ACCESS;
            true
        } else {
            false
        }
    }

    fn drop_write(&mut self, id: ArchetypeComponentId) {
        let id_access = self.access.get_or_insert_with(id, || BASE_ACCESS);
        *id_access = BASE_ACCESS;
    }
}

impl<'w> Drop for WorldCell<'w> {
    fn drop(&mut self) {
        let mut access = self.access.borrow_mut();
        // give world ArchetypeComponentAccess back to reuse allocations
        let _ = std::mem::swap(&mut self.world.archetype_component_access, &mut *access);
    }
}

pub struct WorldBorrow<'w, T> {
    value: &'w T,
    archetype_component_id: ArchetypeComponentId,
    access: Rc<RefCell<ArchetypeComponentAccess>>,
}

impl<'w, T> WorldBorrow<'w, T> {
    fn new(
        value: &'w T,
        archetype_component_id: ArchetypeComponentId,
        access: Rc<RefCell<ArchetypeComponentAccess>>,
    ) -> Self {
        if !access.borrow_mut().read(archetype_component_id) {
            panic!(
                "Attempted to immutably access {}, but it is already mutably borrowed",
                std::any::type_name::<T>()
            )
        }
        Self {
            value,
            archetype_component_id,
            access,
        }
    }
}

impl<'w, T> Deref for WorldBorrow<'w, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T> Drop for WorldBorrow<'w, T> {
    fn drop(&mut self) {
        let mut access = self.access.borrow_mut();
        access.drop_read(self.archetype_component_id);
    }
}

pub struct WorldBorrowMut<'w, T> {
    value: Mut<'w, T>,
    archetype_component_id: ArchetypeComponentId,
    access: Rc<RefCell<ArchetypeComponentAccess>>,
}

impl<'w, T> WorldBorrowMut<'w, T> {
    fn new(
        value: Mut<'w, T>,
        archetype_component_id: ArchetypeComponentId,
        access: Rc<RefCell<ArchetypeComponentAccess>>,
    ) -> Self {
        if !access.borrow_mut().write(archetype_component_id) {
            panic!(
                "Attempted to mutably access {}, but it is already mutably borrowed",
                std::any::type_name::<T>()
            )
        }
        Self {
            value,
            archetype_component_id,
            access,
        }
    }
}

impl<'w, T> Deref for WorldBorrowMut<'w, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<'w, T> DerefMut for WorldBorrowMut<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.value
    }
}

impl<'w, T> Drop for WorldBorrowMut<'w, T> {
    fn drop(&mut self) {
        let mut access = self.access.borrow_mut();
        access.drop_write(self.archetype_component_id);
    }
}

impl<'w> WorldCell<'w> {
    pub(crate) fn new(world: &'w mut World) -> Self {
        // this is cheap because ArchetypeComponentAccess::new() is const / allocation free
        let access = std::mem::replace(
            &mut world.archetype_component_access,
            ArchetypeComponentAccess::new(),
        );
        // world's ArchetypeComponentAccess is recycled to cut down on allocations
        Self {
            world,
            access: Rc::new(RefCell::new(access)),
        }
    }

    pub fn get_resource<T: Resource>(&self) -> Option<WorldBorrow<'_, T>> {
        let component_id = self.world.components.get_resource_id(TypeId::of::<T>())?;
        let resource_archetype = self.world.archetypes.resource();
        let archetype_component_id = resource_archetype.get_archetype_component_id(component_id)?;
        Some(WorldBorrow::new(
            // SAFE: ComponentId matches TypeId
            unsafe { self.world.get_resource_with_id(component_id)? },
            archetype_component_id,
            self.access.clone(),
        ))
    }

    pub fn get_resource_mut<T: Resource>(&self) -> Option<WorldBorrowMut<'_, T>> {
        let component_id = self.world.components.get_resource_id(TypeId::of::<T>())?;
        let resource_archetype = self.world.archetypes.resource();
        let archetype_component_id = resource_archetype.get_archetype_component_id(component_id)?;
        Some(WorldBorrowMut::new(
            // SAFE: ComponentId matches TypeId and access is checked by WorldBorrowMut
            unsafe {
                self.world
                    .get_resource_unchecked_mut_with_id(component_id)?
            },
            archetype_component_id,
            self.access.clone(),
        ))
    }

    pub fn get_non_send<T: 'static>(&self) -> Option<WorldBorrow<'_, T>> {
        let component_id = self.world.components.get_resource_id(TypeId::of::<T>())?;
        let resource_archetype = self.world.archetypes.resource();
        let archetype_component_id = resource_archetype.get_archetype_component_id(component_id)?;
        Some(WorldBorrow::new(
            // SAFE: ComponentId matches TypeId
            unsafe { self.world.get_non_send_with_id(component_id)? },
            archetype_component_id,
            self.access.clone(),
        ))
    }

    pub fn get_non_send_mut<T: 'static>(&self) -> Option<WorldBorrowMut<'_, T>> {
        let component_id = self.world.components.get_resource_id(TypeId::of::<T>())?;
        let resource_archetype = self.world.archetypes.resource();
        let archetype_component_id = resource_archetype.get_archetype_component_id(component_id)?;
        Some(WorldBorrowMut::new(
            // SAFE: ComponentId matches TypeId and access is checked by WorldBorrowMut
            unsafe {
                self.world
                    .get_non_send_unchecked_mut_with_id(component_id)?
            },
            archetype_component_id,
            self.access.clone(),
        ))
    }
}