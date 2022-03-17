#![cfg_attr(
    target_arch = "spirv",
    no_std,
    feature(register_attr),
    register_attr(spirv)
)]

use spirv_std::glam::{vec4, Vec2, Vec3, Vec4};
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

pub struct PushConstants {
    pub offset: Vec2
}

#[spirv(fragment)]
pub fn main_fs(
    frag_color: Vec3,
    output: &mut Vec4
) {
    *output = vec4(frag_color.x, frag_color.y, frag_color.z, 1.0);
}

#[spirv(vertex)]
pub fn main_vs(
    in_pos: Vec2,
    in_color: Vec3,
    #[spirv(push_constant)] push: &PushConstants,
    #[spirv(position, invariant)] out_pos: &mut Vec4,
    frag_color: &mut Vec3,
) {
    *out_pos = vec4(in_pos.x + push.offset.x, in_pos.y + push.offset.y, 0.0, 1.0);
    *frag_color = in_color;
}
