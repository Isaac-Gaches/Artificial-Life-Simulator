use wgpu::{BindingType, Buffer, Device, ShaderModule,CommandEncoder};

pub struct Compute{
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group: wgpu::BindGroup,
}

impl Compute{
    pub fn new(device: &Device,instance_buffer: &Buffer,shader: &ShaderModule)->Self{

        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("Compute Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry{
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some("Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry{
                binding: 1,
                resource: instance_buffer.as_entire_binding(),
            }],
        });

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor{
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: shader,
            entry_point: "cs_main",
        });

        Self{
            compute_bind_group,
            compute_pipeline
        }
    }
    pub fn compute_pass(&self,encoder: &mut CommandEncoder<>){
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.compute_bind_group,&[]);
        compute_pass.dispatch_workgroups(10000,1,1);
    }
}