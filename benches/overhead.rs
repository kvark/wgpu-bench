/*!
    Benchmark of CPU function overhead.
!*/

#[macro_use]
extern crate criterion;

use futures::executor;
use std::iter;

fn init_adapter() -> wgpu::Adapter {
    let instance = wgpu::Instance::new();
    let adapter_future = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
            compatible_surface: None,
        },
        wgpu::UnsafeExtensions::disallow(),
        wgpu::BackendBit::PRIMARY,
    );
    executor::block_on(adapter_future).unwrap()
}

fn init_device() -> (wgpu::Device, wgpu::Queue) {
    let adapter = init_adapter();
    let device_future = adapter.request_device(&wgpu::DeviceDescriptor::default(), None);
    executor::block_on(device_future).unwrap()
}

fn initialization(c: &mut criterion::Criterion) {
    let adapter = init_adapter();

    //TODO: requires proper device destruction
    if false {
        c.bench_function("Adapter::request_device", |b| {
            b.iter(|| {
                let device_future =
                    adapter.request_device(&wgpu::DeviceDescriptor::default(), None);
                let _ = executor::block_on(device_future).unwrap();
            })
        });
    }
}

fn resource_creation(c: &mut criterion::Criterion) {
    let (device, _) = init_device();

    //Warning: Metal/Intel hangs after creating 200k objects

    c.bench_function("Device::create_buffer", |b| {
        let desc = wgpu::BufferDescriptor {
            label: None,
            size: 16,
            usage: wgpu::BufferUsage::VERTEX,
            mapped_at_creation: false,
        };
        b.iter(|| {
            let _ = device.create_buffer(&desc);
        });
        device.poll(wgpu::Maintain::Wait);
    });

    c.bench_function("Device::create_texture", |b| {
        let desc = wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsage::SAMPLED,
        };

        b.iter(|| {
            let _ = device.create_texture(&desc);
        });
        device.poll(wgpu::Maintain::Wait);
    });

    c.bench_function("Device::create_sampler", |b| {
        let desc = wgpu::SamplerDescriptor::default();
        b.iter(|| {
            let _ = device.create_sampler(&desc);
        });
        device.poll(wgpu::Maintain::Wait);
    });
}

fn command_encoding(c: &mut criterion::Criterion) {
    let (device, queue) = init_device();

    let buffer_size = 16;
    let buffer_desc = wgpu::BufferDescriptor {
        label: None,
        size: buffer_size,
        usage: wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::COPY_DST,
        mapped_at_creation: false,
    };
    let texture_desc = wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 4,
            height: 4,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    let texture = device.create_texture(&texture_desc);
    let pass_desc = wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: &texture.create_default_view(),
            resolve_target: None,
            load_op: wgpu::LoadOp::Clear,
            store_op: wgpu::StoreOp::Store,
            clear_color: wgpu::Color::BLACK,
        }],
        depth_stencil_attachment: None,
    };

    //Warning: each render pass ends up creating a new command buffer,
    // thus hitting the queue limit of command buffers eventually.

    c.bench_function("CommandEncoder::begin_render_pass", |b| {
        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        b.iter(|| {
            let _ = command_encoder.begin_render_pass(&pass_desc);
        });
        queue.submit(iter::once(command_encoder.finish()));
    });

    c.bench_function("CommandEncoder::begin_compute_pass", |b| {
        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        b.iter(|| {
            let _ = command_encoder.begin_compute_pass();
        });
        queue.submit(iter::once(command_encoder.finish()));
    });

    //Warning: if too much work is submitted, GPU will time out.

    c.bench_function("CommandEncoder::copy_buffer_to_buffer", |b| {
        let buf_src = device.create_buffer(&buffer_desc);
        let buf_dst = device.create_buffer(&buffer_desc);
        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        b.iter(|| {
            command_encoder.copy_buffer_to_buffer(&buf_src, 0, &buf_dst, 0, buffer_size);
        });
        queue.submit(iter::once(command_encoder.finish()));
    });
}

fn queue_operation(c: &mut criterion::Criterion) {
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
    let (device, queue) = executor::block_on(device_future).unwrap();

    c.bench_function("Queue::submit(empty)", |b| {
        b.iter(|| {
            queue.submit(None);
        })
    });

    c.bench_function("Queue::submit(dummy_command_buffer)", |b| {
        b.iter(|| {
            let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            queue.submit(iter::once(encoder.finish()));
        })
    });
}

criterion_group!(
    name = overhead;
    config = criterion::Criterion
        ::default()
        .warm_up_time(std::time::Duration::from_millis(200))
        .measurement_time(std::time::Duration::from_millis(1000))
        .sample_size(50);
    targets = initialization, resource_creation, command_encoding, queue_operation
);
criterion_main!(overhead);
