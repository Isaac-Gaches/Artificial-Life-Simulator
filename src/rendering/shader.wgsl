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
    ratio: f32,
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
    out.clip_position = vec4<f32>(((v*instance.scale+vec3<f32>(instance.position-camera.position,0.)))*camera.zoom, 1.0) * vec4<f32>(camera.ratio,1.0,1.0,1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

struct CircleVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

@vertex
fn vs_circle(
    vertex: VertexInput,
    instance: InstanceInput,
) -> CircleVertexOutput {
    var out: CircleVertexOutput;
    out.vert_position = vertex.position;
    out.color = instance.color;
    var cs: f32 = cos(instance.rotation);
    var sn: f32 = sin(instance.rotation);
    var v: vec3<f32> = vec3<f32>((vertex.position.x - 0.5) * cs - (vertex.position.y - 0.5) * sn,(vertex.position.x - 0.5) * sn + (vertex.position.y - 0.5) * cs,vertex.position.z);
    out.clip_position = vec4<f32>(((v*instance.scale+vec3<f32>(instance.position-camera.position,0.)))*camera.zoom, 1.0) * vec4<f32>(camera.ratio,1.0,1.0,1.0);
    return out;
}

@fragment
fn fs_circle(in: CircleVertexOutput) -> @location(0) vec4<f32> {
    if (in.vert_position.x - 0.5) * (in.vert_position.x - 0.5) + (in.vert_position.y - 0.5) * (in.vert_position.y - 0.5) > 0.25{
        return vec4<f32>(0.0,0.,0.,0.);
    }
    return vec4<f32>(in.color, 1.0);
}