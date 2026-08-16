use std::io::Cursor;
use std::io::{Read, Seek};
use std::fs::File;
use anyhow::*;

pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

// TODO: Read mipmaps, cube maps, arrays, etc.
// https://docs.rs/dds/latest/dds/
impl Texture {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8], 
        label: &str
    ) -> Result<Self> {
        let cursor = Cursor::new(bytes);
        Self::from_readable(device, queue, cursor, Some(label))
    }

    pub fn from_file(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        filepath: &str,
        label: &str
    ) -> Result<Self> {
        let file = File::open(filepath).unwrap();
        Self::from_readable(device, queue, &file, Some(label))
    }

    pub fn from_readable<T>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        source: T,
        label: Option<&str>
    ) -> Result<Self>
    where
        T: Read + Seek,
    {
        let mut decoder = dds::Decoder::new(source).unwrap();
        // ensure the file contains a single texture
        assert!(decoder.layout().is_texture());
        // prepare a buffer to decode as 8-bit RGBA
        let size = decoder.main_size();
        let mut buf = vec![0_u8; size.pixels() as usize * 4];
        let view = dds::ImageViewMut::new(&mut buf, size, dds::ColorFormat::RGBA_U8).unwrap();
        let wi = view.width().clone();
        let he = view.height().clone();
        // decode into the buffer
        decoder.read_surface(view).unwrap();

        let rgba = buf.clone();

        let size = wgpu::Extent3d {
            width: wi,
            height: he,
            // All textures are stored as 3D, we represent our 2D texture
            // by setting depth to 1.
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB, so we need to reflect that here.
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                // This is the same as with the SurfaceConfig. It
                // specifies what texture formats can be used to
                // create TextureViews for this texture. The base
                // texture format (Rgba8UnormSrgb in this case) is
                // always supported. Note that using a different
                // texture format is not supported on the WebGL2
                // backend.
                view_formats: &[],
            }
        );

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &rgba,
            // The layout of the texture
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * wi),
                rows_per_image: Some(he),
            },
            size,
        );

        // We don't need to configure the texture view much, so let's
        // let wgpu define it.
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self { texture, view, sampler })
    }
}
