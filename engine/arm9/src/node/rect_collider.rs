use fixed::types::*;
use alloc::vec::Vec;
use crate::{pool::Handle, node::Node, hierarchy::{Hierarchy, HierarchyPoolTrait}};

#[derive(Clone)]
pub struct RectColliderExtension {
    pub node_handle: Handle<Node>,
    pub width: I20F12,
    pub height: I20F12,
    pub intersect_list: Vec<Handle<Node>>, // todo: put this on the stack?
}

pub fn check_collisions(hierarchy: &mut Hierarchy) {
    for col in hierarchy.node_ext_pools.rect_collider_pool.iter_mut() {
        col.intersect_list.clear();
    }
    // Compare every element against every other, without unnecessary checks.
    for i in 0..hierarchy.node_ext_pools.rect_collider_pool.vec_len() {
        if let Some(handle) = hierarchy.node_ext_pools.rect_collider_pool.handle_from_index(i) {
            let (col_t, mut col) = hierarchy.node_ext_pools.rect_collider_pool.take(handle);
            if hierarchy.borrow(col.node_handle).global_enabled {
                for j in i+1..hierarchy.node_ext_pools.rect_collider_pool.vec_len() {
                    if let Some(handle_other) = hierarchy.node_ext_pools.rect_collider_pool.handle_from_index(j) {
                        let (col_other_t, mut col_other) = hierarchy.node_ext_pools.rect_collider_pool.take(handle_other);
                        if hierarchy.borrow(col_other.node_handle).global_enabled {
                            if check_collision(hierarchy, &col, &col_other) {
                                col.intersect_list.push(col_other.node_handle);
                                col_other.intersect_list.push(col.node_handle);
                            }
                        }
                        hierarchy.node_ext_pools.rect_collider_pool.put_back(col_other_t, col_other);
                    }
                }
            }
            hierarchy.node_ext_pools.rect_collider_pool.put_back(col_t, col);
        }
    }
}

fn check_collision(hierarchy: &Hierarchy, col1: &RectColliderExtension, col2: &RectColliderExtension) -> bool {
    let r1 = extents_of_collider(hierarchy, col1);
    let r2 = extents_of_collider(hierarchy, col2);
    !(r1.min_x > r2.max_x || r1.max_x < r2.min_x || r1.min_y > r2.max_y || r1.max_y < r2.min_y)
}

fn extents_of_collider(hierarchy: &Hierarchy, col: &RectColliderExtension) -> RectExtents {
    let node = hierarchy.borrow(col.node_handle);
    RectExtents {
        min_x: node.global_transform.x,
        max_x: node.global_transform.x + col.width,
        min_y: node.global_transform.y,
        max_y: node.global_transform.y + col.height,
    }
}

struct RectExtents {
    min_x: I20F12,
    max_x: I20F12,
    min_y: I20F12,
    max_y: I20F12,
}
