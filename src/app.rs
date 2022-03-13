use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

const WINDOW_TITLE: &'static str = "Magma";
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

pub struct App;

impl App {
    pub fn init_window(event_loop: &EventLoop<()>) -> winit::window::Window {
        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .expect("")
    }

    pub fn main_loop(event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            _ => {}
        });
    }
}
