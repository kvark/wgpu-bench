/*!
    Benchmark of CPU memory and descriptor allocators.
!*/

#[macro_use]
extern crate criterion;

use futures::executor;

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

fn memory(c: &mut criterion::Criterion) {
    let (device, queue) = init();

    c.bench_function("Create and free a list of large GPU-local buffers", |b| {
        b.iter(|| {
            let mut buffers = Vec::new();
            for i in 0..7 {
                buffers.push(device.create_buffer(&wgpu::BufferDescriptor {
                    label: None,
                    size: 1 << (16 + i),
                    usage: wgpu::BufferUsage::VERTEX,
                    mapped_at_creation: false,
                }));
            }
            buffers.clear();
            device.poll(wgpu::Maintain::Wait);
        })
    });

    c.bench_function("Run a number of write_buffer commands", |b| {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 1 << 25,
            usage: wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });
        let data = vec![0xFFu8; 1 << 25];
        b.iter(|| {
            for i in 0..10 {
                queue.write_buffer(&buffer, 0, &data[..1 << (16 + i)])
            }
            queue.submit(None);
        });
        device.poll(wgpu::Maintain::Wait);
    });
}

fn bind_group(c: &mut criterion::Criterion) {
    let (device, _) = init();
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        bindings: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                ..Default::default()
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                ty: wgpu::BindingType::SampledTexture {
                    dimension: wgpu::TextureViewDimension::D2,
                    component_type: wgpu::TextureComponentType::Float,
                    multisampled: false,
                },
                ..Default::default()
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                ty: wgpu::BindingType::Sampler { comparison: false },
                ..Default::default()
            },
        ],
    });
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 16,
        usage: wgpu::BufferUsage::UNIFORM,
        mapped_at_creation: false,
    });
    let texture = device.create_texture(&wgpu::TextureDescriptor {
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
    });
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: None,
        format: wgpu::TextureFormat::R8Unorm,
        dimension: wgpu::TextureViewDimension::D2,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        array_layer_count: 1,
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

    c.bench_function("Create and free a list of bind groups", |b| {
        b.iter(|| {
            let mut groups = Vec::new();
            for _ in 0..100 {
                groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &layout,
                    bindings: &[
                        wgpu::Binding {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(buffer.slice(..)),
                        },
                        wgpu::Binding {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView {
                                view: &texture_view,
                                read_only_depth_stencil: false,
                            },
                        },
                        wgpu::Binding {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                }));
            }
            groups.clear();
            device.poll(wgpu::Maintain::Wait);
        })
    });
}

criterion_group!(
    name = allocation;
    config = criterion::Criterion
        ::default()
        .warm_up_time(std::time::Duration::from_millis(200))
        .measurement_time(std::time::Duration::from_millis(1000))
        .sample_size(10);
    targets = memory, bind_group
);
criterion_main!(allocation);
