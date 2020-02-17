
pub fn texture_from_image_path<F, R>(factory: &mut F, path: &str) -> Result<gfx::handle::ShaderResourceView<R, [f32; 4]>, String>
    where F: gfx::Factory<R>, R: gfx::Resources
{   
    let img = match image::open( path ) {
        Ok( i ) => i.to_rgba(),
        Err( _e ) => return Err( _e.to_string())
    };
    
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2( width as u16, height as u16, gfx::texture::AaMode::Single );

    match factory.create_texture_immutable_u8::<crate::ColorFormat>( kind, gfx::texture::Mipmap::Provided, &[&img] ) {
        Ok( _v ) => Ok( _v.1 ),
        Err( _e ) => Err( _e.to_string())
    }
}