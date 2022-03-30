use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::{Window as WinitWindow, WindowBuilder},
};

pub struct Window {
    window: WinitWindow,
    event_loop: EventLoop<()>,
    should_close: bool,
}

impl Window {
    pub fn new(width: u32, height: u32, title: &'static str) -> Window {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width, height))
            .build(&event_loop)
            .map_err(|err| log::error!("Failed to create window: {}", err))
            .unwrap();

        Window {
            window,
            event_loop,
            should_close: false,
        }
    }

    pub fn poll_events(&mut self) {
        self.event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => self.should_close = true,
                    _ => (),
                },
                Event::MainEventsCleared => *control_flow = ControlFlow::Exit,
                _ => (),
            }
        });
    }

    pub fn should_close(&self) -> bool {
        self.should_close
    }

    pub fn winit(&self) -> &WinitWindow {
        &self.window
    }
}
