struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct InstanceInput {
    @location(1) position: vec2<f32>,
    @location(2) rotation: f32,
    @location(3) scale: f32,
    @location(4) color: vec3<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct Camera{
    position: vec2<f32>,
    zoom: f32,
}

@group(0) @binding(0) var<uniform> camera: Camera;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = instance.color;
    var cs: f32 = cos(instance.rotation);
    var sn: f32 = sin(instance.rotation);
    var v: vec3<f32> = vec3<f32>((vertex.position.x - 0.5) * cs - (vertex.position.y - 0.5) * sn,(vertex.position.x - 0.5) * sn + (vertex.position.y - 0.5) * cs,vertex.position.z);
    out.clip_position = vec4<f32>(((v*instance.scale+vec3<f32>(instance.position-camera.position,0.)))*camera.zoom, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}