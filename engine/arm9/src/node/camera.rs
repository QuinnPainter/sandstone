use crate::{pool::Handle, node::Node, hierarchy::Hierarchy};

pub struct CameraExtension {
    pub node_handle: Handle<Node>,
    pub active_main: bool,
    pub active_sub: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ActiveCameras {
    pub main: Option<Handle<CameraExtension>>,
    pub sub: Option<Handle<CameraExtension>>,
}

pub(crate) struct CameraExtensionHandler {}

impl CameraExtensionHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_active_cameras<'a>(&self, hierarchy: &Hierarchy) -> ActiveCameras {
        let mut active_cams = ActiveCameras {
            main: None,
            sub: None,
        };
        // todo: what to do when multiple active cameras for screen? priority system?
        for i in 0..hierarchy.node_ext_pools.camera_pool.vec_len() {
            if let Some(handle) = hierarchy.node_ext_pools.camera_pool.handle_from_index(i) {
                let cam = hierarchy.node_ext_pools.camera_pool.borrow(handle);
                if cam.active_main { active_cams.main = Some(handle); }
                if cam.active_sub { active_cams.sub = Some(handle); }
            }
        }
        active_cams
    }
}
