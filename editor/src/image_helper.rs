use std::{fs::File, path::Path};
use glow::HasContext;
use imgui_glow_renderer::TextureMap;

pub fn reload_texture(renderer: &mut imgui_glow_renderer::AutoRenderer, tex: imgui::TextureId, path: &Path) -> imgui::TextureId {
    tex
}

pub fn load_texture(renderer: &mut imgui_glow_renderer::AutoRenderer, path: &Path) -> imgui::TextureId {
    let gl = renderer.gl_context();
    let decoder = png::Decoder::new(File::open(path).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let image = {
        let mut buf = vec![0u8; reader.output_buffer_size()];
        reader.next_frame(&mut buf).unwrap();
        buf
    };
    let (width, height) = reader.info().size();

    let gl_texture = unsafe { gl.create_texture() }.expect("unable to create GL texture");

    unsafe {
        gl.bind_texture(glow::TEXTURE_2D, Some(gl_texture));
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
            Some(&image),
        )
    }
    renderer.texture_map_mut().register(gl_texture).unwrap()
}
