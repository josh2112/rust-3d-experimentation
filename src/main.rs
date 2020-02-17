#[macro_use]
extern crate gfx;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_device_gl;

use std::path::Path;

#[macro_use]
extern crate arrayref;

use cgmath::{Matrix4,Vector3,Point3,Deg};
use cgmath::prelude::One;
use cgmath::InnerSpace;
use tobj;

mod utils;

type ColorFormat = gfx::format::Rgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        normal: [f32; 3] = "a_Normal",
        uv: [f32; 2] = "a_Uv",
    }

    constant Transforms {
        model: [[f32; 4]; 4] = "u_Model",
        view: [[f32; 4]; 4] = "u_View",
        proj: [[f32; 4]; 4] = "u_Proj",
    }

    constant Lights {
        light_pos: [f32; 4] = "u_LightPos",
        light_color: [f32; 4] = "u_LightColor",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transforms: gfx::ConstantBuffer<Transforms> = "Transforms",
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        lights: gfx::ConstantBuffer<Lights> = "Lights",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

struct Entity {
    vertices: std::boxed::Box<[Vertex]>,
    indices: Vec<u32>,
    texture: gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
    transform: Matrix4<f32>,
    vertex_buffer: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    slice: gfx::Slice<gfx_device_gl::Resources>
}

impl Entity {
    fn from_model( model_path: &str, texture_path: &str, factory: &mut gfx_device_gl::Factory ) -> Entity {

        let (models, _) = tobj::load_obj( &Path::new( model_path ))
            .expect( "Failed to load model!" );
        let model = models.iter().next().unwrap();

        let vertices = (0..model.mesh.positions.len()/3)
            .map( |n| Vertex {
                pos: array_ref![model.mesh.positions, n*3, 3].clone(),
                normal: array_ref![model.mesh.normals, n*3, 3].clone(),
                uv: array_ref![model.mesh.texcoords, n*2, 2].clone()
            } ).collect::<Vec<Vertex>>().into_boxed_slice();
        let indices = model.mesh.indices.clone();

        let (vbuf, slice) = factory.create_vertex_buffer_with_slice( &vertices, &indices as &[u32] );

        Entity {
            vertices: vertices,
            indices: indices,
            texture: utils::texture_from_image_path( factory, texture_path ).expect( "Failed to load texture image!" ),
            transform: Matrix4::one().into(),
            vertex_buffer: vbuf,
            slice: slice
        }
    }

    fn center( &self ) -> Point3<f32> {
        Point3::new(
            self.vertices.iter().map( |v| v.pos[0] ).sum(),
            self.vertices.iter().map( |v| v.pos[1] ).sum(),
            self.vertices.iter().map( |v| v.pos[2] ).sum())
            / (self.vertices.len() as f32)
    }
}

pub fn main() {

    println!( "Creating window..." );

    let win_builder = glutin::WindowBuilder::new().with_title( "Three-D-Game".to_string() )
        .with_dimensions( glutin::dpi::LogicalSize::new( 800., 600. ));
    let gl_builder = glutin::ContextBuilder::new().with_vsync( true );

    let mut event_loop = glutin::EventsLoop::new();
    
    let (window_ctx, mut device, mut factory, color_view, depth_view) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>( win_builder, gl_builder, &event_loop )
        .expect( "Failed to create window!" );

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let pso = factory.create_pipeline_simple(
        include_bytes!( "../shaders/simple.glslv" ),
        include_bytes!( "../shaders/simple.glslf" ),
        pipe::new()
    ).expect( "Failed to create pipeline state object!" );
    
    let dragon = Entity::from_model( "resources/obj/dragon.obj", "resources/img/white.png", &mut factory );

    let mut transforms = Transforms {
        model: Matrix4::one().into(),
        view: Matrix4::look_at(
            Point3::new(0.0, 7.0, 15.0),
            dragon.center(),
            Vector3::unit_y(),
        ).into(),
        proj: Matrix4::one().into()
    };

    let lights = Lights {
        light_pos: [10.0, 10.0, 10.0, 0.0],
        light_color: [0.0, 0.3, 0.0, 0.0]
    };

    let transforms_buffer = factory.create_constant_buffer( 1 );
    let lights_buffer = factory.create_constant_buffer( 1 );
    let texture_sampler = factory.create_sampler_linear();
    
    let mut data = pipe::Data {
        vbuf: dragon.vertex_buffer,
        transforms: transforms_buffer,
        tex: (dragon.texture, texture_sampler),
        lights: lights_buffer,
        out: color_view,
        out_depth: depth_view,
    };

    let mut angle: f32 = 0.0;

    let mut running = true;
    while running {
        event_loop.poll_events( |event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                use glutin::WindowEvent::*;
                match event {
                    KeyboardInput {
                        input: glutin::KeyboardInput {
                            virtual_keycode: Some( glutin::VirtualKeyCode::Escape ), ..
                        }, ..
                    } | CloseRequested => running = false,
                    Resized( size ) => {
                        window_ctx.resize( size.to_physical( window_ctx.get_hidpi_factor()));
                        gfx_window_glutin::update_views( &window_ctx, &mut data.out, &mut data.out_depth );
                        let aspect = (size.width/size.height) as f32;
                        transforms.proj = cgmath::perspective( Deg(60.0f32), aspect, 0.1, 1000.0 ).into();
                    },
                    _ => (),
                }
            }
        });
        
        encoder.clear( &data.out, [0.2, 0.0, 0.0, 1.0] );
        encoder.clear_depth( &data.out_depth, 1.0 );

        angle += 0.5;
        transforms.model = (Matrix4::from_angle_y( Deg( angle )) * dragon.transform).into();

        encoder.update_buffer( &data.transforms, &[transforms], 0 ).expect( "Failed to update transforms!" );
        encoder.update_buffer( &data.lights, &[lights], 0 ).expect( "Failed to update lights!" );
        encoder.draw( &dragon.slice, &pso, &data );
        
        encoder.flush( &mut device );
        window_ctx.swap_buffers().unwrap();
        device.cleanup();
    }
}