use bytemuck::cast_slice;
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
        self.instances.update();

        let instances_raw = self.instances.get_raw();

        self.queue.write_buffer(&self.instance_buffer, 0, cast_slice(&instances_raw));
        self.queue.write_buffer(&self.camera_configuration.buffer, 0, cast_slice(&[self.camera_configuration.uniform]));
    }
}
