use std::iter::once;

#[allow(clippy::wildcard_imports)]
use wgpu::*;
use wgpu::LoadOp::Clear;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;

use crate::models::Texture;
use crate::state::State;

impl State {
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    #[inline]
    pub fn reconfigure_surface(&mut self) {
        self.surface.configure(&self.device, &self.surface_configuration);
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
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
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(Operations { load: Clear(1.0), store: true }),
                    stencil_ops: None,
                }),
            });

            let geometry = &self.geometry;

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_configuration.bind_group, &[]);
            render_pass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(geometry.index_buffer.slice(..), IndexFormat::Uint16);
            #[allow(clippy::cast_possible_truncation)]
            render_pass.draw_indexed(0..geometry.num_indices, 0, 0..self.instances.len() as _);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(once(encoder.finish()));

        output.present();

        Ok(())
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.surface_configuration.width = new_size.width;
            self.surface_configuration.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_configuration);

            self.depth_texture = Texture::create_depth_texture(
                &self.device,
                &self.surface_configuration,
                "depth_texture",
            );
        }
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_configuration.uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_configuration.buffer, 0, bytemuck::cast_slice(&[self.camera_configuration.uniform]));
    }
}
