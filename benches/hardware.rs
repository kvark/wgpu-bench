#[macro_use]
extern crate criterion;

use futures::executor;
use std::iter;

fn init() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new();
    let adapter_future = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: None,
        },
        wgpu::UnsafeExtensions::disallow(),
        wgpu::BackendBit::PRIMARY,
    );
    let adapter = executor::block_on(adapter_future).unwrap();
    let device_future = adapter.request_device(&wgpu::DeviceDescriptor::default(), None);
    executor::block_on(device_future).unwrap()
}

fn load_shader(name: &str) -> Vec<u32> {
    let ty = if name.ends_with(".vert") {
        glsl_to_spirv::ShaderType::Vertex
    } else if name.ends_with(".frag") {
        glsl_to_spirv::ShaderType::Fragment
    } else if name.ends_with(".comp") {
        glsl_to_spirv::ShaderType::Compute
    } else {
        unreachable!()
    };

    let path = std::path::PathBuf::from("shaders").join(name);
    let code = std::fs::read_to_string(path).unwrap();
    wgpu::read_spirv(glsl_to_spirv::compile(&code, ty).unwrap()).unwrap()
}

fn pixel_write(c: &mut criterion::Criterion) {
    let(device, queue) = init();

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[],
    });
    let vs_bytes = load_shader("quad.vert");
    let fs_bytes = load_shader("white.frag");

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &device.create_shader_module(&vs_bytes),
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &device.create_shader_module(&fs_bytes),
            entry_point: "main",
        }),
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
        color_states: &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Rgba32Float,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 4096,
            height: 4096,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    });
    let pass_desc = wgpu::RenderPassDescriptor {
        color_attachments: &[
            wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &texture.create_default_view(),
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
            },
        ],
        depth_stencil_attachment: None,
    };

    //TODO: takes too long, need GPU timers
    if false {
        c.bench_function("pixel write", |b| b.iter(|| {
            let mut command_encoder = device.create_command_encoder(
                &wgpu::CommandEncoderDescriptor::default()
            );
            {
                let mut pass = command_encoder.begin_render_pass(&pass_desc);
                pass.set_pipeline(&pipeline);
                pass.draw(0..4, 0..200);
            }
            queue.submit(iter::once(command_encoder.finish()));
            device.poll(wgpu::Maintain::Wait);
        }));
    }
}

criterion_group!(
    name = hardware;
    config = criterion::Criterion
        ::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_millis(2000))
        .sample_size(10);
    targets = pixel_write
);
criterion_main!(hardware);
