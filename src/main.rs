#[macro_use] extern crate gfx;

use gfx::traits::FactoryExt;
use gfx::Device;

use cgmath::{Matrix4,Vector3,Point3,Deg};
use cgmath::prelude::One;

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
    }
}

use image;

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
    
    let (window_ctx, mut device, mut factory, mut color_view, mut depth_view) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>( builder, context, &event_loop )
        .expect( "Failed to create window!" );

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let pso = factory.create_pipeline_simple(
        include_bytes!( "../shaders/simple.glslv" ),
        include_bytes!( "../shaders/simple.glslf" ),
        pipe::new()
    ).expect( "Failed to create pipeline state object!" );

    const SQ_VERTICES: &[Vertex] = &[
        Vertex { pos: [0.5, -0.5, 0.0], uv: [1.0, 1.0] },
        Vertex { pos: [0.5, 0.5, 0.0], uv: [1.0, 0.0] },
        Vertex { pos: [-0.5, 0.5, 0.0], uv: [0.0, 0.0] },
        Vertex { pos: [-0.5, -0.5, 0.0], uv: [0.0, 1.0] },
    ];
    const SQ_INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

    let mut locals: Constants = Constants {
        model: Matrix4::one().into(),
        view: Matrix4::look_at(
            Point3::new(0.0, 0.0, 2.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::unit_y(),
        ).into(),
        proj: Matrix4::one().into(),
    };

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice( SQ_VERTICES, SQ_INDICES );
    let locals_buffer = factory.create_constant_buffer( 1 );
    let texture = load_texture( &mut factory, "resources/img/donutface.jpg" ).expect( "Failed to load texture image!" );
    
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        tex: (texture, factory.create_sampler_linear()),
        constants: locals_buffer,
        out: color_view,
    };

    let mut angle: f64 = 0.0;

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
                        gfx_window_glutin::update_views( &window_ctx, &mut data.out, &mut depth_view );
                        
                        let aspect = (size.width/size.height) as f32;
                        locals.proj = cgmath::perspective( Deg(60.0f32), aspect, 0.1, 1000.0).into();
                    },
                    _ => (),
                }
            }
        });

        angle += 0.001;
        let (angle_cos, angle_sin) = (angle.cos() as f32, angle.sin() as f32);
        locals.model[0][0] = angle_cos;
        locals.model[1][1] = angle_cos;
        locals.model[0][1] = angle_sin;
        locals.model[1][0] = -angle_sin;

        encoder.clear( &data.out, [0.2, 0.0, 0.0, 1.0] );
        encoder.update_buffer( &data.constants, &[locals], 0 ).expect( "Failed to update transform!" );
        encoder.draw( &slice, &pso, &data );
        
        encoder.flush( &mut device );
        window_ctx.swap_buffers().unwrap();
        device.cleanup();
    }
}