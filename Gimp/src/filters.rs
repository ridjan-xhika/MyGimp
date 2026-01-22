use serde::{Deserialize, Serialize};
use wgpu;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterType {
    GaussianBlur,
    UnsharpMask,
    ColorAdjust,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct FilterParams {
    pub filter_type: FilterType,
    pub blur_radius: f32,       // For Gaussian Blur (1.0 to 16.0)
    pub sharpen_strength: f32,  // For Unsharp Mask (0.0 to 2.0)
    pub brightness: f32,        // For Color Adjust (-1.0 to 1.0)
    pub contrast: f32,          // For Color Adjust (0.5 to 2.0)
    pub saturation: f32,        // For Color Adjust (0.0 to 2.0)
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            filter_type: FilterType::GaussianBlur,
            blur_radius: 2.0,
            sharpen_strength: 0.5,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
        }
    }
}

pub struct FilterPipeline {
    pub gaussian_blur_pipeline: wgpu::RenderPipeline,
    pub unsharp_mask_pipeline: wgpu::RenderPipeline,
    pub color_adjust_pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub params_buffer: wgpu::Buffer,
}

impl FilterPipeline {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Filter Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Filter Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create simple vertex shader (shared by all filters)
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Filter Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("filter_vertex.wgsl"))),
        });

        // Create fragment shaders for each filter type
        let gaussian_blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gaussian Blur Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("gaussian_blur.wgsl"))),
        });

        let unsharp_mask_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Unsharp Mask Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("unsharp_mask.wgsl"))),
        });

        let color_adjust_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Color Adjust Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("color_adjust.wgsl"))),
        });

        let gaussian_blur_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Gaussian Blur Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &gaussian_blur_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        let unsharp_mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Unsharp Mask Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &unsharp_mask_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        let color_adjust_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Color Adjust Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &color_adjust_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        // Create params uniform buffer
        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Filter Params Buffer"),
            size: 32, // 4 * 4 bytes for 4 f32 values
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            gaussian_blur_pipeline,
            unsharp_mask_pipeline,
            color_adjust_pipeline,
            bind_group_layout,
            params_buffer,
        }
    }

    pub fn update_params(&self, queue: &wgpu::Queue, params: &FilterParams) {
        let data = [
            params.blur_radius.to_bits(),
            params.sharpen_strength.to_bits(),
            params.brightness.to_bits(),
            params.contrast.to_bits(),
        ];
        let bytes = unsafe {
            std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(&data))
        };
        queue.write_buffer(&self.params_buffer, 0, bytes);
    }

    pub fn get_pipeline(&self, filter_type: FilterType) -> &wgpu::RenderPipeline {
        match filter_type {
            FilterType::GaussianBlur => &self.gaussian_blur_pipeline,
            FilterType::UnsharpMask => &self.unsharp_mask_pipeline,
            FilterType::ColorAdjust => &self.color_adjust_pipeline,
        }
    }
}
