use super::TransformObject;
use crate::{
    classes::{component::Component, p_ptr::PPtr},
    until::binrw_parser::{Quat, Vec3},
    SerializedFileMetadata,
};
use binrw::binrw;
use glam::Mat4;
use supercow::Supercow;

#[binrw]
#[brw(import_raw(args: SerializedFileMetadata))]
#[derive(Debug)]
pub struct Transform {
    #[brw(args_raw = args.clone())]
    component: Component,
    local_rotation: Quat,
    local_position: Vec3,
    local_scale: Vec3,
    children_count: i32,
    #[br(count = children_count,args { inner: args.clone() })]
    #[bw(args_raw = args.clone())]
    children: Vec<PPtr>,
    #[brw(args_raw = args)]
    father: PPtr,
}

impl TransformObject for Transform {
    fn get_game_object(&self) -> Supercow<PPtr> {
        self.component.get_game_object()
    }

    fn get_father(&self) -> Supercow<PPtr> {
        Supercow::borrowed(&self.father)
    }

    fn get_local_mat(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            *self.local_scale,
            *self.local_rotation,
            *self.local_position,
        )
    }
}
