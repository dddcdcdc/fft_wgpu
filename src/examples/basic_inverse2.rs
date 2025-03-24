use num_complex::Complex32 as Complex;

#[tokio::main]
async fn main() {
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::default();

    // `request_adapter` instantiates the general connection to the GPU
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await
        .unwrap();

    dbg!(adapter.limits());

    // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
    //  `features` being the available features.
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: adapter.features(),
                required_limits: adapter.limits(),
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();

    let data = vec![Complex::new(1.0, 0.0); 512 * 500 * 5];
    let len = data.len();

    // let mut data_cpu = data
    //     .iter()
    //     .map(|c| rustfft::num_complex::Complex::new(c.re, 0.0))
    //     .collect::<Vec<_>>();
    // let fft = rustfft::FftPlanner::new().plan_fft_forward(16);
    // fft.process(&mut data_cpu);
    // fft.process(&mut data_cpu);
    // println!("{:?}", &data_cpu[..]);

    let mut ans = vec![Complex::ZERO; len];

    // Instantiates buffer without data.
    // `usage` of buffer specifies how it can be used:
    //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
    //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (len * std::mem::size_of::<Complex>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let src = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (len * std::mem::size_of::<Complex>()) as u64,
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (len * std::mem::size_of::<Complex>()) as u64,
        usage: wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let fft_onlyinverse = fft_wgpu::Onlyinverse::new(&device, &queue, &src,&buffer_b,512);
    // let fft_forward_2 = fft_wgpu::Forward::new(&device, &queue, &src, 16);
    let normalize = fft_wgpu::Normalize::new(&device, &queue, &src, &buffer_b,512);
    let timer = std::time::Instant::now();

    for _ in 0..1000 {
        queue.write_buffer(&src, 0, bytemuck::cast_slice(data.as_slice()));
        // A command encoder executes one or many pipelines.
        // It is to WebGPU what a command buffer is to Vulkan.
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let _output = fft_onlyinverse.proc(&mut encoder);
        // let output = fft_forward.proc(&mut encoder);
        //let output = fft_forward_2.proc(&mut encoder);

        let output2 = normalize.proc(&mut encoder);

        encoder.copy_buffer_to_buffer(
            output2,
            0,
            &staging_buffer,
            0,
            (len * std::mem::size_of::<Complex>()) as u64,
        );
        queue.submit(Some(encoder.finish()));

        // let rn = fft_forward.round_num.slice(..);

        // rn.map_async(wgpu::MapMode::Read, move |_| {});

        // device.poll(wgpu::Maintain::wait()).panic_on_timeout();
        // let a: Vec<u8> = rn.get_mapped_range().iter().copied().collect();
        // dbg!(a);
        // fft_forward.round_num.unmap();

        // Note that we're not calling `.await` here.
        let buffer_slice = staging_buffer.slice(..);

        buffer_slice.map_async(wgpu::MapMode::Read, move |_| {});

        device.poll(wgpu::Maintain::wait()).panic_on_timeout();

        // Gets contents of buffer
        let data = buffer_slice.get_mapped_range();

        // // Since contents are got in bytes, this converts these bytes back to u32
        bytemuck::cast_slice(&data).clone_into(&mut ans);

        // println!("{:?}", &ans[..512]);

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        staging_buffer.unmap(); // Unmaps buffer from memory
        // If you are familiar with C++ these 2 lines can be thought of similarly to:
        //   delete myPointer;
        //   myPointer = NULL;
        // It effectively frees the memory
    }
    dbg!(timer.elapsed());
}

mod test {
    use num_complex::Complex32 as Complex;
    use rustfft::{FftDirection, FftPlanner};

    #[tokio::test]
    // 在main函数末尾添加以下测试代码
    async fn test_ifft() {
        let instance = wgpu::Instance::default();
        // `request_adapter` instantiates the general connection to the GPU
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .unwrap();
        dbg!(adapter.limits());
        // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
        //  `features` being the available features.
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: adapter.features(),
                    required_limits: adapter.limits(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();
        let data = vec![Complex::new(2.1327392395, 3.033729); 512 * 500 * 5];
        let len = data.len();
        // let mut data_cpu = data
        //     .iter()
        //     .map(|c| rustfft::num_complex::Complex::new(c.re, 0.0))
        //     .collect::<Vec<_>>();
        // let fft = rustfft::FftPlanner::new().plan_fft_forward(16);
        // fft.process(&mut data_cpu);
        // fft.process(&mut data_cpu);
        // println!("{:?}", &data_cpu[..]);
        let mut ans = vec![Complex::ZERO; len];
        // Instantiates buffer without data.
        // `usage` of buffer specifies how it can be used:
        //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
        //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (len * std::mem::size_of::<Complex>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let src = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (len * std::mem::size_of::<Complex>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let src2 = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (len * std::mem::size_of::<Complex>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let fft_onlyinverse = fft_wgpu::Onlyinverse::new(&device, &queue, &src,&src2, 512);
        // let fft_forward_2 = fft_wgpu::Forward::new(&device, &queue, &src, 16);
        let normalize = fft_wgpu::Normalize::new(&device, &queue, &src,&src2, 512);
        let timer = std::time::Instant::now();
        for _ in 0..1 {
            queue.write_buffer(&src, 0, bytemuck::cast_slice(data.as_slice()));
            // A command encoder executes one or many pipelines.
            // It is to WebGPU what a command buffer is to Vulkan.
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let _output = fft_onlyinverse.proc(&mut encoder);
            // let output = fft_forward.proc(&mut encoder);
            //let output = fft_forward_2.proc(&mut encoder);
            let output2 = normalize.proc(&mut encoder);
            encoder.copy_buffer_to_buffer(
                output2,
                0,
                &staging_buffer,
                0,
                (len * std::mem::size_of::<Complex>()) as u64,
            );
            queue.submit(Some(encoder.finish()));
            // Note that we're not calling `.await` here.
            let buffer_slice = staging_buffer.slice(..);
            buffer_slice.map_async(wgpu::MapMode::Read, move |_| {});
            device.poll(wgpu::Maintain::wait()).panic_on_timeout();
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // // Since contents are got in bytes, this converts these bytes back to u32
            bytemuck::cast_slice(&data).clone_into(&mut ans);
            //  println!("{:?}", &ans[..10]);
            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(data);
            staging_buffer.unmap(); // Unmaps buffer from memory
            // If you are familiar with C++ these 2 lines can be thought of similarly to:
            //   delete myPointer;
            //   myPointer = NULL;
            // It effectively frees the memory
        }
        dbg!(timer.elapsed());

        // 生成参考结果
        let mut reference = vec![Complex::new(2.0, 1.0); len];
        let mut planner = FftPlanner::<f32>::new();
        let inverse_plan = planner.plan_fft(512, FftDirection::Inverse);

        // 对每个512元素的块进行处理
        data.chunks_exact(512)
            .zip(reference.chunks_exact_mut(512))
            .for_each(|(input_chunk, output_chunk)| {
                let mut buffer = input_chunk.to_vec();
                inverse_plan.process(&mut buffer);

                // 归一化处理，因为rustfft的逆FFT不自动缩放
                for c in &mut buffer {
                    *c /= 512.0;
                }

                output_chunk.copy_from_slice(&buffer);
            });

        // 比较结果
        let max_error = ans
            .iter()
            .zip(reference.iter())
            .map(|(a, r)| {
                let re_diff = (a.re - r.re).abs();
                let im_diff = (a.im - r.im).abs();
                re_diff.max(im_diff)
            })
            .fold(0.0f32, |max, diff| max.max(diff));

        println!("最大误差: {}", max_error);
        assert!(
            max_error < 1e-5,
            "WebGPU结果与参考结果不一致，最大误差: {}",
            max_error
        );
    }
}
