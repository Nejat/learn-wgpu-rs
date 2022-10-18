use std::iter::once;

#[allow(clippy::wildcard_imports)]
use wgpu::*;
use wgpu::LoadOp::Clear;

use crate::state::State;

pub fn render(state: &mut State) -> Result<(), SurfaceError> {
    let output = state.surface.get_current_texture()?;
    let view = output.texture.create_view(&TextureViewDescriptor::default());

    let mut encoder = state.device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("render encoder"),
    });

    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: Clear(Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        let geometry = &state.geometry;

        render_pass.set_pipeline(&state.render_pipeline);
        render_pass.set_bind_group(0, &state.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &state.camera_configuration.bind_group, &[]);
        render_pass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, state.instance_buffer.slice(..));
        render_pass.set_index_buffer(geometry.index_buffer.slice(..), IndexFormat::Uint16);
        #[allow(clippy::cast_possible_truncation)]
        render_pass.draw_indexed(0..geometry.num_indices, 0, 0..state.instances.len() as _);
    }

    // submit will accept anything that implements IntoIter
    state.queue.submit(once(encoder.finish()));

    output.present();

    Ok(())
}
