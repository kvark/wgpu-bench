#[macro_use]
extern crate criterion;

fn initialization(c: &mut criterion::Criterion) {
    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions::default(),
        wgpu::BackendBit::PRIMARY,
    ).unwrap();

    //TODO: requires proper device destruction
    if false {
        c.bench_function("Adapter::request_device", |b| b.iter(|| {
            let _ = adapter.request_device(&wgpu::DeviceDescriptor::default());
        }));
    }
}

fn resource_creation(c: &mut criterion::Criterion) {
    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions::default(),
        wgpu::BackendBit::PRIMARY,
    ).unwrap();
    let (device, _) = adapter.request_device(&wgpu::DeviceDescriptor::default());

    //TODO: this hangs at 200k buffers on Metal/Intel
    if false {
        let desc = wgpu::BufferDescriptor {
            size: 16,
            usage: wgpu::BufferUsage::VERTEX,
        };
        c.bench_function("Device::create_buffer", |b| b.iter(|| {
            let _ = device.create_buffer(&desc);
        }));
        c.bench_function("Device::create_buffer_mapped", |b| b.iter(|| {
            let _ = device.create_buffer_mapped(16, wgpu::BufferUsage::VERTEX).finish();
        }));
    }
    //TODO: also hanges on Metal/Intel
    if false {
        let desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsage::SAMPLED,
        };
        c.bench_function("Device::create_texture", |b| b.iter(|| {
            let _ = device.create_texture(&desc);
        }));
    }

    if true {
        let desc = wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 10.0,
            compare_function: wgpu::CompareFunction::Always,
        };

        c.bench_function("Device::create_sampler", |b| b.iter(|| {
            let _ = device.create_sampler(&desc);
        }));
    }
}

fn command_encoding(c: &mut criterion::Criterion) {
    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions::default(),
        wgpu::BackendBit::PRIMARY,
    ).unwrap();
    let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor::default());

    let buffer_size = 16;
    let buffer_desc = wgpu::BufferDescriptor {
        size: buffer_size,
        usage: wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::COPY_DST,
    };
    let texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: 4,
            height: 4,
            depth: 1,
        },
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
    };
    let texture = device.create_texture(&texture_desc);
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

    //TODO: requires creation of `MTLCommandBuffer` per pass, runs out of the queue limits
    if false {
        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        c.bench_function("CommandEncoder::begin_render_pass", |b| b.iter(|| {
            let _ = command_encoder.begin_render_pass(&pass_desc);
        }));

        queue.submit(&[command_encoder.finish()]);
    }
    if true {
        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        c.bench_function("CommandEncoder::begin_compute_pass", |b| b.iter(|| {
            let _ = command_encoder.begin_compute_pass();
        }));

        queue.submit(&[command_encoder.finish()]);
    }

    //TODO: takes too much time, need to make the timeout configurable
    if false {
        let buf_src = device.create_buffer(&buffer_desc);
        let buf_dst = device.create_buffer(&buffer_desc);
        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        c.bench_function("CommandEncoder::copy_buffer_to_buffer", |b| b.iter(|| {
            command_encoder.copy_buffer_to_buffer(&buf_src, 0, &buf_dst, 0, buffer_size);
        }));

        queue.submit(&[command_encoder.finish()]);
    }
}

fn queue_operation(c: &mut criterion::Criterion) {
    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions::default(),
        wgpu::BackendBit::PRIMARY,
    ).unwrap();
    let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor::default());

    c.bench_function("Queue::submit(empty)", |b| b.iter(|| {
        queue.submit(&[]);
    }));

    c.bench_function("Queue::submit(dummy_command_buffer)", |b| b.iter(|| {
        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        queue.submit(&[encoder.finish()]);
    }));
}

criterion_group!(overhead, initialization, resource_creation, command_encoding, queue_operation);
criterion_main!(overhead);