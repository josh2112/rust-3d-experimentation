#[macro_use] extern crate gfx;

use gfx::traits::FactoryExt;
use gfx::Device;

use std::path::Path;

#[macro_use]
extern crate arrayref;

use cgmath::{Matrix4,Vector3,Point3,Deg};
use cgmath::prelude::One;
use cgmath::InnerSpace;
use image;
use tobj;

type ColorFormat = gfx::format::Rgba8;
type DepthFormat = gfx::format::DepthStencil;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
    }

    constant Constants {
        model: [[f32; 4]; 4] = "u_Model",
        view: [[f32; 4]; 4] = "u_View",
        proj: [[f32; 4]; 4] = "u_Proj",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        constants: gfx::ConstantBuffer<Constants> = "Constants",
        tex: gfx::TextureSampler<[f32; 4]> = "t_Texture",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
        out_depth: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

fn load_texture<F, R>(factory: &mut F, path: &str) -> Result<gfx::handle::ShaderResourceView<R, [f32; 4]>, String>
    where F: gfx::Factory<R>, R: gfx::Resources
{   
    let img = match image::open( path ) {
        Ok( i ) => i.to_rgba(),
        Err( _e ) => return Err( _e.to_string())
    };
    
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2( width as u16, height as u16, gfx::texture::AaMode::Single );

    match factory.create_texture_immutable_u8::<ColorFormat>( kind, gfx::texture::Mipmap::Provided, &[&img] ) {
        Ok( _v ) => Ok( _v.1 ),
        Err( _e ) => Err( _e.to_string())
    }
}

pub fn main() {
    let builder = glutin::WindowBuilder::new()
        .with_title("Three-D-Game".to_string())
        .with_dimensions( glutin::dpi::LogicalSize::new( 800., 600. ));
    
    let context = glutin::ContextBuilder::new().with_vsync( true );
    let mut event_loop = glutin::EventsLoop::new();
    
    let (window_ctx, mut device, mut factory, color_view, depth_view) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>( builder, context, &event_loop )
        .expect( "Failed to create window!" );

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let pso = factory.create_pipeline_simple(
        include_bytes!( "../shaders/simple.glslv" ),
        include_bytes!( "../shaders/simple.glslf" ),
        pipe::new()
    ).expect( "Failed to create pipeline state object!" );

    let (models, _) = tobj::load_obj( &Path::new( "resources/obj/stall.obj"))
        .expect( "Failed to load model!" );
    let stall_model = models.iter().next().unwrap();

    let stall_vertices = (0..stall_model.mesh.positions.len()/3)
        .map( |n| Vertex {
            pos: array_ref![stall_model.mesh.positions, n*3, 3].clone(),
            uv: array_ref![stall_model.mesh.texcoords, n*2, 2].clone()
        } ).collect::<Vec<Vertex>>().into_boxed_slice();
        
    let mut locals: Constants = Constants {
        model: Matrix4::one().into(),
        view: Matrix4::look_at(
            Point3::new(10.0, 10.0, -10.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        ).into(),
        proj: Matrix4::one().into(),
    };
    

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice( &stall_vertices, &stall_model.mesh.indices as &[u32] );
    let locals_buffer = factory.create_constant_buffer( 1 );
    let texture = load_texture( &mut factory, "resources/img/stall.png" ).expect( "Failed to load texture image!" );
    
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        tex: (texture, factory.create_sampler_linear()),
        constants: locals_buffer,
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
                        locals.proj = cgmath::perspective( Deg(60.0f32), aspect, 0.1, 1000.0 ).into();
                    },
                    _ => (),
                }
            }
        });

        angle += 0.5;
        //locals.model = Matrix4::from_axis_angle( Vector3::new( 1f32, 1f32, 1f32 ).normalize(), Deg(angle) ).into();
        //locals.model = Matrix4::from_angle_z( Deg( angle )).into();

        encoder.clear( &data.out, [0.2, 0.0, 0.0, 1.0] );
        encoder.clear_depth( &data.out_depth, 1.0 );
        encoder.update_buffer( &data.constants, &[locals], 0 ).expect( "Failed to update transform!" );
        encoder.draw( &slice, &pso, &data );
        
        encoder.flush( &mut device );
        window_ctx.swap_buffers().unwrap();
        device.cleanup();
    }
}