use cgmath::{InnerSpace, SquareMatrix};
use wgpu::{util::DeviceExt, Buffer};
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{Key, NamedKey},
};

pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    buffer: Buffer,
    uniform: CameraUniform,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub controller: CameraController,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

impl Camera {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // 2.
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
    pub fn new(device: &wgpu::Device, width: f32, height: f32) -> Self {
        let controller = CameraController::new(0.01);
        let mut uniform = CameraUniform::new();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        let camera = Camera {
            eye: (0.0, 5.0, 10.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: width / height,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            bind_group,
            bind_group_layout,
            controller,
            buffer,
            uniform,
        };
        uniform.update_view_proj(&camera);

        camera
    }

    pub fn update(&mut self, queue: &mut wgpu::Queue) {
        let controller = &self.controller;
        let forward = (self.target - self.eye).normalize();
        if controller.is_forward_pressed {
            self.eye += forward * controller.speed;
        }
        if controller.is_backward_pressed {
            self.eye -= forward * controller.speed;
        }

        let right = forward.cross(self.up);

        if controller.is_right_pressed {
            self.eye += right * controller.speed;
        }
        if controller.is_left_pressed {
            self.eye -= right * controller.speed;
        }
        let pm = self.build_view_projection_matrix().into();
        self.uniform.view_proj = pm;

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                let is_pressed = event.state == ElementState::Pressed;
                match event.logical_key {
                    Key::Named(NamedKey::Space) => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    Key::Named(NamedKey::Shift) => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
