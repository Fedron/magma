//! Demonstrates how you can render a 3D cube with perspective

use std::rc::Rc;

use anyhow::Result;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use magma::prelude::*;

#[repr(C)]
#[derive(Vertex)]
struct CubeVertex {
    #[location = 0]
    position: [f32; 3],
    #[location = 1]
    color: [f32; 3],
}

// Our push constant will contain the model, view, projection matrix which will be calculated on
// the host
#[derive(UniformBuffer)]
#[ubo(stage = "vertex")]
struct PushConstant {
    _transform_matrix: glam::Mat4,
}

// Wraps a projection and view matrix which will allow us to see the cube with perspective in 3D
struct Camera {
    projection_matrix: glam::Mat4,
    view_matrix: glam::Mat4,
}

impl Camera {
    pub fn new(fovy: f32, aspect: f32, near: f32, far: f32) -> Camera {
        Camera {
            projection_matrix: glam::Mat4::perspective_rh(fovy, aspect, near, far),
            view_matrix: glam::Mat4::IDENTITY,
        }
    }

    pub fn look_at(&mut self, eye: glam::Vec3, target: glam::Vec3) {
        self.view_matrix = glam::Mat4::look_at_rh(eye, target, glam::Vec3::Y);
    }
}

// A transform will allow us to easily create a model matrix from a position, rotation and scale
struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Vec3,
    pub scale: glam::Vec3,
}

impl Transform {
    pub fn as_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale,
            glam::Quat::from_euler(
                glam::EulerRot::YXZ,
                self.rotation.y,
                self.rotation.x,
                self.rotation.z,
            ),
            self.position,
        )
    }
}

fn main() -> Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Cube").build(&event_loop)?;

    let instance = Instance::new(&[DebugLayer::KhronosValidation])?;
    let physical_device = PhysicalDevice::builder()
        .preferred_type(PhysicalDeviceType::CPU)
        .add_queue_family(QueueFamily::new(QueueFlags::GRAPHICS))
        .device_extensions(&[DeviceExtension::Swapchain])
        .build(&instance)?;
    let logical_device = Rc::new(LogicalDevice::new(instance, physical_device)?);

    let mut surface = Surface::new(
        logical_device.instance(),
        logical_device.physical_device(),
        &window,
    )?;
    let mut swapchain = Swapchain::builder()
        .preferred_color_format(ColorFormat::Srgb)
        .preferred_present_mode(PresentMode::Mailbox)
        .build(logical_device.clone(), &surface)?;

    let vertex_shader = Shader::new("shaders/cube.vert")?;
    let fragment_shader = Shader::new("shaders/cube.frag")?;

    let pipeline = Pipeline::<CubeVertex, PushConstant>::builder()
        .attach_shader(vertex_shader)
        .attach_shader(fragment_shader)
        .render_pass(swapchain.render_pass())
        .build(logical_device.clone())?;

    let mut command_pool = CommandPool::new(
        logical_device.clone(),
        logical_device
            .physical_device()
            .queue_family(QueueFlags::GRAPHICS)
            .unwrap(),
    )?;
    command_pool.allocate_buffers(
        swapchain.framebuffers().len() as u32,
        CommandBufferLevel::Primary,
    )?;

    let mut staging_buffer = Buffer::<CubeVertex, 8>::new(
        logical_device.clone(),
        BufferUsageFlags::TRANSFER_SRC,
        MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
    )?;
    staging_buffer.map(u64::MAX, 0)?;
    staging_buffer.write(&[
        CubeVertex {
            // Bottom-back-left
            position: [-0.5, 0.5, -0.5],
            color: [1.0, 0.0, 0.0],
        },
        CubeVertex {
            // Bottom-back-right
            position: [0.5, 0.5, -0.5],
            color: [0.0, 1.0, 0.0],
        },
        CubeVertex {
            // Bottom-front-right
            position: [0.5, 0.5, 0.5],
            color: [0.0, 0.0, 1.0],
        },
        CubeVertex {
            // Bottom-front-left
            position: [-0.5, 0.5, 0.5],
            color: [1.0, 1.0, 0.0],
        },
        CubeVertex {
            // Top-back-left
            position: [-0.5, -0.5, -0.5],
            color: [0.0, 1.0, 1.0],
        },
        CubeVertex {
            // Top-back-right
            position: [0.5, -0.5, -0.5],
            color: [1.0, 0.0, 1.0],
        },
        CubeVertex {
            // Top-front-right
            position: [0.5, -0.5, 0.5],
            color: [0.0, 0.0, 0.0],
        },
        CubeVertex {
            // Top-front-left
            position: [-0.5, -0.5, 0.5],
            color: [1.0, 1.0, 1.0],
        },
    ]);

    let mut vertex_buffer = Buffer::<CubeVertex, 8>::new(
        logical_device.clone(),
        BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
        MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    vertex_buffer.copy_from(&staging_buffer, &command_pool)?;

    let mut staging_buffer = Buffer::<u32, 36>::new(
        logical_device.clone(),
        BufferUsageFlags::TRANSFER_SRC,
        MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
    )?;
    staging_buffer.map(u64::MAX, 0)?;
    staging_buffer.write(&[
        4, 0, 3, 4, 3, 7, // Left face
        5, 2, 1, 5, 6, 2, // Right face
        7, 2, 3, 7, 6, 2, // Front face
        5, 0, 1, 5, 4, 0, // Back face
        4, 6, 7, 4, 5, 6, // Top face
        1, 3, 2, 1, 0, 3, // Bottom face
    ]);

    let mut index_buffer = Buffer::<u32, 36>::new(
        logical_device.clone(),
        BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::INDEX_BUFFER,
        MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    index_buffer.copy_from(&staging_buffer, &command_pool)?;

    let mut should_close = false;
    let mut is_minimized = false;

    let mut camera = Camera::new(50_f32.to_radians(), swapchain.aspect_ratio(), 0.1, 10.0);
    let mut cube_transform = Transform {
        position: glam::Vec3::ZERO,
        rotation: glam::Vec3::ZERO,
        scale: glam::Vec3::ONE,
    };

    while !should_close {
        event_loop.run_return(|event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => should_close = true,
                WindowEvent::Resized(size) => {
                    if size.width == 0 && size.height == 0 {
                        is_minimized = true
                    } else {
                        is_minimized = false
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => *control_flow = ControlFlow::Exit,
            _ => {}
        });

        camera.look_at(glam::vec3(-3.0, 0.0, 0.0), glam::Vec3::ZERO);
        cube_transform.rotation.x += 0.001;
        cube_transform.rotation.y += 0.002;
        cube_transform.rotation.z += 0.003;

        if is_minimized {
            continue;
        }

        let result = swapchain.acquire_next_image();
        if result.is_err() {
            swapchain = recreate_swapchain(
                swapchain,
                logical_device.clone(),
                &mut surface,
                &mut command_pool,
            )?;
            continue;
        }
        let image_index = result.unwrap();

        let command_buffer = command_pool.buffers_mut().get_mut(image_index).unwrap();
        command_buffer.begin()?;
        command_buffer.set_clear_color((0.01, 0.01, 0.01));
        command_buffer.begin_render_pass(
            swapchain.render_pass(),
            *swapchain.framebuffers().get(image_index).unwrap(),
            swapchain.extent(),
        )?;

        let extent = swapchain.extent();
        command_buffer.set_viewport(extent.0 as f32, extent.1 as f32)?;
        command_buffer.set_scissor(extent.clone())?;

        command_buffer.bind_pipeline(&pipeline);
        command_buffer.bind_vertex_buffer(&vertex_buffer);
        command_buffer.bind_index_buffer(&index_buffer);

        pipeline.set_push_constant(
            &command_buffer,
            PushConstant {
                _transform_matrix: camera.projection_matrix
                    * camera.view_matrix
                    * cube_transform.as_matrix(),
            },
        );

        command_buffer.draw_indexed(36, 1, 0, 0, 0);

        command_buffer.end_render_pass();
        command_buffer.end()?;

        let result = swapchain.submit_command_buffer(command_buffer, image_index);
        if result.is_err() {
            swapchain = recreate_swapchain(
                swapchain,
                logical_device.clone(),
                &mut surface,
                &mut command_pool,
            )?;
            continue;
        }
    }

    logical_device.wait_for_idle()?;

    Ok(())
}

fn recreate_swapchain(
    old_swapchain: Swapchain,
    logical_device: Rc<LogicalDevice>,
    surface: &mut Surface,
    command_pool: &mut CommandPool,
) -> Result<Swapchain> {
    logical_device.wait_for_idle()?;
    surface.update(logical_device.physical_device())?;

    let swapchain = Swapchain::builder()
        .old_swapchain(old_swapchain)
        .preferred_color_format(ColorFormat::Srgb)
        .preferred_present_mode(PresentMode::Mailbox)
        .build(logical_device.clone(), surface)?;

    if swapchain.framebuffers().len() != command_pool.buffers().len() {
        command_pool.free_buffers();
        command_pool.allocate_buffers(
            swapchain.framebuffers().len() as u32,
            CommandBufferLevel::Primary,
        )?;
    }

    Ok(swapchain)
}
