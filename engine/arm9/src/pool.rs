use core::marker::PhantomData;
use core::num::NonZeroU32;
use core::ops::{Index, IndexMut};
use core::fmt::Debug;
use alloc::vec::Vec;

pub struct Pool<T> {
    data_vec: Vec<PoolEntry<T>>,
    // Contains the indices of all the freed / empty entries.
    free_stack: Vec<usize>
}

struct PoolEntry<T> {
    generation: NonZeroU32,
    data: Option<T>
}

#[derive(PartialEq, Eq)]
pub struct Handle<T> {
    index: usize,
    generation: NonZeroU32, // making this nonzero makes Option<Handle> more efficient using niche optimisation
    // Doesn't do anything, only serves to make Handles of different types incompatible with each other
    phantom_type: PhantomData<T>
}

unsafe impl<T> Send for Handle<T> {}
unsafe impl<T> Sync for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            generation: self.generation,
            phantom_type: PhantomData::<T>
        }
    }
}
impl<T> Copy for Handle<T> {}

impl<T> Debug for Handle<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Handle")
            .field("index", &self.index)
            .field("generation", &self.generation)
            .finish()
    }
}

pub struct Ticket<T> {
    index: usize,
    phantom_type: PhantomData<T>
}

impl<T> Pool<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data_vec: Vec::new(),
            free_stack: Vec::new()
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn vec_len(&self) -> usize {
        self.data_vec.len()
    }

    #[inline]
    #[must_use]
    pub fn borrow(&self, handle: Handle<T>) -> &T {
        self.try_borrow(handle).expect(alloc::format!("Tried to borrow from pool with an invalid handle: {:?}", handle).as_str())
    }

    #[inline]
    #[must_use]
    pub fn try_borrow(&self, handle: Handle<T>) -> Option<&T> {
        let entry = self.data_vec.get(handle.index)?;
        if entry.generation == handle.generation {
            entry.data.as_ref()
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn borrow_mut(&mut self, handle: Handle<T>) -> &mut T {
        self.try_borrow_mut(handle).expect(alloc::format!("Tried to mutably borrow from pool with an invalid handle: {:?}", handle).as_str())
    }

    #[inline]
    #[must_use]
    pub fn try_borrow_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        let entry = self.data_vec.get_mut(handle.index)?;
        if entry.generation == handle.generation {
            entry.data.as_mut()
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn take(&mut self, handle: Handle<T>) -> (Ticket<T>, T) {
        self.try_take(handle).expect(alloc::format!("Tried to take from pool with an invalid handle: {:?}", handle).as_str())
    }

    #[inline]
    #[must_use]
    pub fn try_take(&mut self, handle: Handle<T>) -> Option<(Ticket<T>, T)> {
        let entry = self.data_vec.get_mut(handle.index)?;
        if entry.generation == handle.generation {
            let taken_data = entry.data.take()?;
            Some((Ticket { index: handle.index, phantom_type: PhantomData::<T> }, taken_data))
        } else {
            None
        }
    }

    // try_take without generation. Used by hierarchy update loop.
    #[inline]
    #[must_use]
    pub(crate) fn try_take_by_index(&mut self, index: usize) -> Option<(Ticket<T>, T)> {
        let entry = self.data_vec.get_mut(index)?;
        let taken_data = entry.data.take()?;
        Some((Ticket { index: index, phantom_type: PhantomData::<T> }, taken_data))
    }

    #[inline]
    pub fn put_back(&mut self, ticket: Ticket<T>, value: T) -> Handle<T> {
        let record = if cfg!(debug_assertions) {
            self.data_vec.get_mut(ticket.index).expect("Tried to put back out-of-bounds ticket index")
        } else {
            unsafe { self.data_vec.get_unchecked_mut(ticket.index) }
        };
        let old = record.data.replace(value);
        debug_assert!(old.is_none(), "Tried to put back to an occupied slot");
        Handle {
            index: ticket.index,
            generation: record.generation,
            phantom_type: PhantomData::<T>
        }
    }

    pub fn add(&mut self, data: T) -> Handle<T> {
        if let Some(index) = self.free_stack.pop() {
            let entry = if cfg!(debug_assertions) {
                self.data_vec.get_mut(index).expect("Out-of-bounds index in pool's free stack")
            } else {
                unsafe { self.data_vec.get_unchecked_mut(index) }
            };

            debug_assert!(entry.data.is_none(), "Tried to add object to slot in pool that is already occupied");

            entry.generation = if cfg!(debug_assertions) { 
                entry.generation.checked_add(1).expect("Pool generation number overflow")
            } else {
                unsafe { entry.generation.unchecked_add(1) }
            };
            entry.data.replace(data);

            Handle {
                index: index,
                generation: entry.generation,
                phantom_type: PhantomData::<T>
            }
        } else {
            let new_generation = NonZeroU32::new(1).unwrap();
            self.data_vec.push(PoolEntry {
                generation: new_generation,
                data: Some(data)
            });

            Handle {
                index: self.data_vec.len() - 1,
                generation: new_generation,
                phantom_type: PhantomData::<T>
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> PoolIterator<T> {
        unsafe {
            PoolIterator {
                ptr: self.data_vec.as_ptr(),
                end: self.data_vec.as_ptr().add(self.data_vec.len()),
                phantom_type: PhantomData::<&T>
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn iter_mut(&mut self) -> PoolIteratorMut<T> {
        unsafe {
            PoolIteratorMut {
                ptr: self.data_vec.as_mut_ptr(),
                end: self.data_vec.as_mut_ptr().add(self.data_vec.len()),
                phantom_type: PhantomData::<&mut T>
            }
        }
    }
}

impl<T> Index<Handle<T>> for Pool<T> {
    type Output = T;
    fn index(&self, index: Handle<T>) -> &Self::Output {
        self.borrow(index)
    }
}

impl<T> IndexMut<Handle<T>> for Pool<T> {
    fn index_mut(&mut self, index: Handle<T>) -> &mut Self::Output {
        self.borrow_mut(index)
    }
}

impl<'a, T> IntoIterator for &'a Pool<T> {
    type Item = &'a T;
    type IntoIter = PoolIterator<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Pool<T> {
    type Item = &'a mut T;
    type IntoIter = PoolIteratorMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct PoolIterator<'a, T> {
    ptr: *const PoolEntry<T>,
    end: *const PoolEntry<T>,
    phantom_type: PhantomData<&'a T>
}

impl<'a, T> Iterator for PoolIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            while self.ptr != self.end {
                let current_entry = &*self.ptr;
                self.ptr = self.ptr.offset(1);
                if let Some(data) = current_entry.data.as_ref() {
                    return Some(data);
                }
            }
            None
        }
    }
}

pub struct PoolIteratorMut<'a, T> {
    ptr: *mut PoolEntry<T>,
    end: *mut PoolEntry<T>,
    phantom_type: PhantomData<&'a mut T>
}

impl<'a, T> Iterator for PoolIteratorMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            while self.ptr != self.end {
                let current_entry = &mut *self.ptr;
                self.ptr = self.ptr.offset(1);
                if let Some(data) = current_entry.data.as_mut() {
                    return Some(data);
                }
            }
            None
        }
    }
}
