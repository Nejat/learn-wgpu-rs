use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

#[cfg(target_arch = "wasm32")]
pub fn initialize_canvas(window: &Window) {
    use winit::dpi::PhysicalSize;
    use winit::platform::web::WindowExtWebSys;

    // Winit prevents sizing with CSS, so we have to set
    // the size manually when on web.
    window.set_inner_size(PhysicalSize::new(450, 400));

    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("lean-wgpu")?;
            let canvas = web_sys::Element::from(window.canvas());

            dst.append_child(&canvas).ok()?;

            Some(())
        })
        .expect("Couldn't append canvas to document body.");
}

pub fn initialize_environment() -> (EventLoop<()>, Window) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("learn-wgpu")
        .build(&event_loop)
        .unwrap();

    (event_loop, window)
}

pub fn initialize_logging() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_error_panic_hook::set_once();
            tracing_wasm::set_as_global_default();
        } else {
            tracing_subscriber::fmt::init();
        }
    }
}
