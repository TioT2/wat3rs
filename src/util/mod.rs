pub mod math;
pub mod obj;
pub mod rand;

pub type Vec2f = math::Vec2<f32>;
pub type Vec3f = math::Vec3<f32>;
pub type Vec4f = math::Vec4<f32>;
pub type Ext2f = math::Ext2<f32>;
pub type Mat4x4f = math::Mat4x4<f32>;

unsafe impl bytemuck::NoUninit for Mat4x4f {}
unsafe impl bytemuck::Zeroable for Mat4x4f {}

unsafe impl bytemuck::NoUninit for Vec2f {}
unsafe impl bytemuck::Zeroable for Vec2f {}

unsafe impl bytemuck::NoUninit for Vec3f {}
unsafe impl bytemuck::Zeroable for Vec3f {}

unsafe impl bytemuck::NoUninit for Vec4f {}
unsafe impl bytemuck::Zeroable for Vec4f {}
