use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;

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
        }
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_configuration.uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_configuration.buffer, 0, bytemuck::cast_slice(&[self.camera_configuration.uniform]));
    }
}
