#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]

#![allow(clippy::module_name_repetitions)]

#[macro_use]
extern crate bytemuck;
#[macro_use]
extern crate tracing;
#[macro_use]
extern crate wgpu;

use wgpu::SurfaceError;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::init::{initialize_environment, initialize_logging};
#[cfg(target_arch = "wasm32")]
use crate::init::initialize_canvas;
use crate::state::{render, State};

mod init;
mod meshes;
mod models;
mod state;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
#[allow(clippy::future_not_send)] // todo: winit event loop is not send
pub async fn run() {
    initialize_logging();
    let (event_loop, window) = initialize_environment();

    #[cfg(target_arch = "wasm32")]
    initialize_canvas(&window);

    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() && !state.input(event) => match event {
                WindowEvent::CloseRequested |
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) =>
                    state.resize(*physical_size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } =>
                    state.resize(**new_inner_size),
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();

                match render(&mut state) {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(SurfaceError::Lost) => state.reconfigure_surface(),
                    // The system is out of memory, we should probably quit
                    Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => error!("{:?}", e),
                }
            }
            Event::MainEventsCleared =>
            // RedrawRequested will only trigger once, unless we manually request it.
                window.request_redraw(),
            _ => {}
        }
    });
}
