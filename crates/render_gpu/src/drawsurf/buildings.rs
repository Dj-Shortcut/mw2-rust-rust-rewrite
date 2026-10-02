use super::exact_pipeline::ExactPipelineRegistry;
use super::scene_depth::{SCENE_DEPTH_FORMAT, SceneDepthTexture};
use bevy::core_pipeline::{Core3d, Core3dSystems};
use bevy::prelude::*;
use bevy::render::render_resource::binding_types::uniform_buffer_sized;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery};
use bevy::render::view::{ExtractedView, Msaa, ViewTarget};
use bevy::render::{Render, RenderApp, RenderSystems};
use rust_building::{BuildingWorld, Grade};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct BuildingFrame {
    pub world: BuildingWorld,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
    normal: [f32; 3],
    grade: f32,
}

#[derive(Resource, Default)]
struct BuildingGpu {
    world: Option<BuildingWorld>,
    vertices: Option<Buffer>,
    indices: Option<Buffer>,
    count: u32,
    view: Option<Buffer>,
    bind: Option<BindGroup>,
    pipelines: HashMap<(TextureFormat, u32), RenderPipeline>,
}

pub(super) fn register(app: &mut App) {
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render
            .init_resource::<BuildingFrame>()
            .init_resource::<BuildingGpu>()
            .add_systems(Render, prepare.in_set(RenderSystems::PrepareResources))
            .add_systems(
                Core3d,
                draw.in_set(Core3dSystems::MainPass)
                    .after(super::draw::ExactColourDrawSet),
            );
    }
}

fn layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "building_material",
        &[uniform_buffer_sized(false, std::num::NonZeroU64::new(64))
            .build(0, ShaderStages::VERTEX)],
    )
}

fn mesh(world: &BuildingWorld) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for p in world.pieces() {
        let grade = match p.grade {
            Grade::Wood => 0.,
            Grade::Stone => 1.,
            Grade::Metal => 2.,
        };
        for (lo, hi) in world.bounds(p) {
            for (axis, u, v) in [(0, 1, 2), (1, 2, 0), (2, 0, 1)] {
                for side in [false, true] {
                    let mut normal = [0.; 3];
                    normal[axis] = if side { 1. } else { -1. };
                    let start = vertices.len() as u32;
                    for (a, b) in [(0., 0.), (1., 0.), (1., 1.), (0., 1.)] {
                        let mut position = lo;
                        position[axis] = if side { hi[axis] } else { lo[axis] };
                        position[u] = lo[u] + a * (hi[u] - lo[u]);
                        position[v] = lo[v] + b * (hi[v] - lo[v]);
                        vertices.push(Vertex {
                            position,
                            uv: [a * (hi[u] - lo[u]) / 120., b * (hi[v] - lo[v]) / 120.],
                            normal,
                            grade,
                        });
                    }
                    indices.extend([start, start + 1, start + 2, start, start + 2, start + 3]);
                }
            }
        }
    }
    (vertices, indices)
}

fn prepare(
    frame: Res<BuildingFrame>,
    published: Res<super::PublishedRenderFrame>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    registry: Res<ExactPipelineRegistry>,
    mut gpu: ResMut<BuildingGpu>,
) {
    if gpu.world.as_ref() != Some(&frame.world) {
        let (vertices, indices) = mesh(&frame.world);
        gpu.count = indices.len() as u32;
        gpu.vertices = (!vertices.is_empty()).then(|| {
            device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("building_vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: BufferUsages::VERTEX,
            })
        });
        gpu.indices = (!indices.is_empty()).then(|| {
            device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("building_indices"),
                contents: bytemuck::cast_slice(&indices),
                usage: BufferUsages::INDEX,
            })
        });
        gpu.world = Some(frame.world.clone());
    }
    if gpu.count == 0 {
        return;
    }
    if gpu.view.is_none() {
        gpu.view = Some(device.create_buffer(&BufferDescriptor {
            label: Some("building_view"),
            size: 64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }
    if gpu.bind.is_none() {
        let layout = registry.bind_group_layout(&device, &layout());
        gpu.bind = Some(device.create_bind_group(
            "building_material",
            &layout,
            &[BindGroupEntry {
                binding: 0,
                resource: gpu.view.as_ref().unwrap().as_entire_binding(),
            }],
        ));
    }
    if let Some(matrix) = published.exec_frame.clip_from_world {
        queue.write_buffer(
            gpu.view.as_ref().unwrap(),
            0,
            bytemuck::cast_slice(&matrix.to_cols_array()),
        );
    }
}

fn pipeline(
    device: &RenderDevice,
    registry: &ExactPipelineRegistry,
    format: TextureFormat,
    samples: u32,
) -> RenderPipeline {
    let shader = unsafe {
        device.create_shader_module(ShaderModuleDescriptor {
            label: Some("building_shader"),
            source: ShaderSource::Wgsl(SHADER.into()),
        })
    };
    let layout = registry.bind_group_layout(device, &layout());
    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("building_pipeline"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let attributes = [
        VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 0,
            shader_location: 0,
        },
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: 12,
            shader_location: 1,
        },
        VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 20,
            shader_location: 2,
        },
        VertexAttribute {
            format: VertexFormat::Float32,
            offset: 32,
            shader_location: 3,
        },
    ];
    device.create_render_pipeline(&RawRenderPipelineDescriptor {
        label: Some("building_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: RawVertexState {
            module: &shader,
            entry_point: Some("vertex"),
            buffers: &[RawVertexBufferLayout {
                array_stride: 36,
                step_mode: VertexStepMode::Vertex,
                attributes: &attributes,
            }],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(RawFragmentState {
            module: &shader,
            entry_point: Some("fragment"),
            targets: &[Some(ColorTargetState {
                format,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: PrimitiveState {
            cull_mode: None,
            ..default()
        },
        depth_stencil: Some(DepthStencilState {
            format: SCENE_DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(CompareFunction::GreaterEqual),
            stencil: default(),
            bias: default(),
        }),
        multisample: MultisampleState {
            count: samples,
            ..default()
        },
        multiview_mask: None,
        cache: None,
    })
}

fn draw(
    view: ViewQuery<(
        &ViewTarget,
        &SceneDepthTexture,
        &ExtractedView,
        Option<&Msaa>,
    )>,
    registry: Res<ExactPipelineRegistry>,
    device: Res<RenderDevice>,
    mut gpu: ResMut<BuildingGpu>,
    mut context: RenderContext,
) {
    if gpu.count == 0 {
        return;
    }
    let (target, depth, extracted, msaa) = view.into_inner();
    let key = (target.main_texture_format(), msaa.map_or(1, Msaa::samples));
    let pipeline = gpu
        .pipelines
        .entry(key)
        .or_insert_with(|| pipeline(&device, &registry, key.0, key.1))
        .clone();
    let (Some(bind), Some(vertices), Some(indices)) = (&gpu.bind, &gpu.vertices, &gpu.indices)
    else {
        return;
    };
    let attachments = [Some(target.get_color_attachment())];
    let mut pass = context.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("building_geometry"),
        color_attachments: &attachments,
        depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    let vp = extracted.viewport;
    let (near, far) =
        super::depth_range::reverse_z_viewport_depth(super::depth_range::GFX_DEPTH_RANGE_SCENE);
    pass.set_viewport(
        vp.x as f32,
        vp.y as f32,
        vp.z as f32,
        vp.w as f32,
        near,
        far,
    );
    pass.set_render_pipeline(&pipeline);
    pass.set_bind_group(0, bind, &[]);
    pass.set_vertex_buffer(0, vertices.slice(..));
    pass.set_index_buffer(indices.slice(..), IndexFormat::Uint32);
    pass.draw_indexed(0..gpu.count, 0, 0..1);
}

pub const SHADER: &str = r#"
@group(0) @binding(0) var<uniform> clip_from_world: mat4x4<f32>;
struct Out {
    @builtin(position) clip:vec4<f32>,
    @location(0) uv:vec2<f32>,
    @location(1) normal:vec3<f32>,
    @location(2) @interpolate(flat) grade:f32,
    @location(3) world:vec3<f32>,
}
@vertex fn vertex(@location(0) position:vec3<f32>,@location(1) uv:vec2<f32>,@location(2) normal:vec3<f32>,@location(3) grade:f32)->Out {
    var out:Out;
    out.clip=clip_from_world*vec4<f32>(position,1.0);
    out.uv=uv;out.normal=normal;out.grade=grade;out.world=position;
    return out;
}
fn hash(p:vec2<f32>)->f32 {
    let q=fract(p*vec2<f32>(0.1031,0.11369));
    let h=q+vec2<f32>(dot(q,q.yx+vec2<f32>(19.19)));
    return fract(h.x*h.y*(h.x+h.y));
}
fn noise(p:vec2<f32>)->f32 {
    let cell=floor(p);let f=fract(p);let s=f*f*(vec2<f32>(3.0)-2.0*f);
    return mix(mix(hash(cell),hash(cell+vec2<f32>(1.0,0.0)),s.x),
        mix(hash(cell+vec2<f32>(0.0,1.0)),hash(cell+vec2<f32>(1.0)),s.x),s.y);
}
fn wood(uv:vec2<f32>,world:vec3<f32>,footprint:f32)->vec3<f32> {
    let row=floor(uv.y*5.0);
    let plank=hash(vec2<f32>(row,floor(world.z/120.0)+floor(world.x/120.0)));
    let warp=noise(uv*vec2<f32>(2.0,11.0)+vec2<f32>(plank*10.0));
    let phase=uv.y*260.0+warp*12.0;
    let fine=(0.5+0.5*sin(phase))*(1.0-clamp(footprint*90.0,0.0,1.0));
    let grain=noise(uv*vec2<f32>(4.0,100.0)+vec2<f32>(plank*14.0));
    let seam=min(fract(uv.y*5.0),1.0-fract(uv.y*5.0));
    let joint=smoothstep(0.018,0.032+footprint,seam);
    let tone=clamp(0.2+0.45*warp+0.22*grain+0.12*plank-0.09*fine,0.0,1.0);
    return mix(vec3<f32>(0.038,0.017,0.007),
        mix(vec3<f32>(0.18,0.073,0.027),vec3<f32>(0.49,0.29,0.13),tone),joint);
}
fn stone(uv:vec2<f32>,world:vec3<f32>,footprint:f32)->vec3<f32> {
    let row=floor(uv.y*3.0);
    let grid=vec2<f32>(uv.x*2.8+select(0.0,0.5,fract(row*0.5)>0.1),uv.y*3.0);
    let cell=floor(grid);let f=fract(grid);
    let edge=min(min(f.x,1.0-f.x),min(f.y,1.0-f.y));
    let erosion=noise(grid*9.0)*0.012;
    let block=smoothstep(0.02+erosion,0.05+footprint*3.0,edge);
    let variation=hash(cell+floor(world.xy/360.0));
    let coarse=noise(grid*5.0+vec2<f32>(variation*17.0));
    let pores=noise(grid*48.0)*(1.0-clamp(footprint*45.0,0.0,1.0));
    let rock=mix(vec3<f32>(0.15,0.17,0.18),vec3<f32>(0.43,0.42,0.37),variation*0.45+coarse*0.4+pores*0.15);
    return mix(vec3<f32>(0.07,0.065,0.055),rock,block);
}
fn metal(uv:vec2<f32>,world:vec3<f32>,footprint:f32)->vec3<f32> {
    let grid=uv*vec2<f32>(2.0,3.0);let f=fract(grid);
    let edge=min(min(f.x,1.0-f.x),min(f.y,1.0-f.y));
    let panel=smoothstep(0.007,0.02+footprint*3.0,edge);
    let corner=min(f,vec2<f32>(1.0)-f)-vec2<f32>(0.12);
    let rivet=1.0-smoothstep(0.028,0.044+footprint*3.0,length(corner));
    let variation=hash(floor(grid)+floor(world.xy/240.0));
    let brushed=(noise(uv*vec2<f32>(2.0,190.0))-0.5)*0.035*(1.0-clamp(footprint*100.0,0.0,1.0));
    let plate=vec3<f32>(0.20,0.24,0.27)+vec3<f32>(variation*0.05+brushed);
    let corrosion=noise(grid*7.0+vec2<f32>(13.0))*0.05*(1.0-panel);
    return mix(vec3<f32>(0.07,0.09,0.10)+vec3<f32>(corrosion,corrosion*0.35,0.0),plate,panel)
        +rivet*vec3<f32>(0.10,0.11,0.12);
}
@fragment fn fragment(in:Out)->@location(0) vec4<f32> {
    let footprint=max(length(dpdx(in.uv)),length(dpdy(in.uv)));
    var albedo:vec3<f32>;
    if in.grade>1.5 {albedo=metal(in.uv,in.world,footprint);}
    else if in.grade>0.5 {albedo=stone(in.uv,in.world,footprint);}
    else {albedo=wood(in.uv,in.world,footprint);}
    let light=0.45+0.55*max(dot(normalize(in.normal),normalize(vec3<f32>(0.35,-0.4,0.85))),0.0);
    return vec4<f32>(albedo*light,1.0);
}
"#;
