use std::path::Path;
use glow::HasContext;
use image::EncodableLayout;
use imgui_glow_renderer::TextureMap;

pub fn load_texture(renderer: &mut imgui_glow_renderer::AutoRenderer, tex: Option<imgui::TextureId>, path: &Path) -> imgui::TextureId {
    let gl = renderer.gl_context();
    
    // Texture has been loaded already - should be unloaded before making a new texture
    if let Some(tex) = tex {
        unsafe { gl.delete_texture(renderer.texture_map().gl_texture(tex).unwrap()); }
    }
    
    let reader = image::io::Reader::open(path).unwrap();
    let image = reader.decode().unwrap();
    let image = image.into_rgba8();

    let (width, height) = image.dimensions();
    let image = image.as_bytes();

    let gl_texture = unsafe { gl.create_texture() }.expect("unable to create GL texture");

    unsafe {
        gl.bind_texture(glow::TEXTURE_2D, Some(gl_texture));
        // These parameters determine how the image will be scaled.
        // They are required (image will not display without setting them)
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as _,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as _,
        );
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as _,
            width as _,
            height as _,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(image),
        )
    }
    renderer.texture_map_mut().register(gl_texture).unwrap()
}
