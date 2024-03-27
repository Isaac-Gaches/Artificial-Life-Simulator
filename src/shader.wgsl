struct VertexInput {
    @location(0) vertex_position: vec2<f32>,
    @location(1) position: vec2<f32>,
    @location(2) color: vec3<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};
struct Camera{
    position: vec2<f32>,
}
@group(0) @binding(0) var<uniform> camera: Camera;
@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = in.color;
    out.clip_position = vec4<f32>(in.vertex_position*0.01+in.position+camera.position,0.0, 1.0);
    return out;
}
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
struct Instance{
    position: vec2<f32>,
    color: vec3<f32>,
}
@group(0) @binding(1) var<storage,read_write> instances: array<Instance>;
@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>){
   var i = instances[global_id.x];
   i.position.x += 0.002;
   instances[global_id.x] = i;
}