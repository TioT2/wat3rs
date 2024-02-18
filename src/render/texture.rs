/// WAT3RS Project
/// `File` render/texture.rs
/// `Description` Rendertexture impementation module
/// `Author` TioT2
/// `Last changed` 17.02.2024

use super::kernel::Kernel;

/// Texture representation structure
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
} // struct Texture

impl Texture {
    pub(super) fn get_texture<'a>(&'a self) -> &'a wgpu::Texture {
        &self.texture
    } // fn get_texture

    pub(super) fn get_view<'a>(&'a self) -> &'a wgpu::TextureView {
        &self.view
    } // fn get_texture
} // impl Texture

impl Kernel {
    pub fn create_texture(&self, descriptor: &wgpu::TextureDescriptor) -> Texture {
        let texture = self.device.create_texture(descriptor);

        Texture {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            texture,
        }
    } // fn create_texture
}