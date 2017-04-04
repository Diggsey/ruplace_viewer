#[macro_use]
extern crate gfx;
extern crate gfx_app;
extern crate winit;
extern crate reqwest;

use std::mem;
use std::fs::File;
use std::io::Read;

use gfx_app::ColorFormat;
use gfx::{Bundle, texture, SHADER_RESOURCE, TRANSFER_DST, TRANSFER_SRC};
use gfx::memory::Usage;
use gfx::handle::Texture;
use gfx::format::Swizzle;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        tex_coord: [f32; 2] = "a_TexCoord",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        color: gfx::TextureSampler<[f32; 4]> = "t_Color",
        out_color: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}


impl Vertex {
    fn new(p: [i8; 3], t: [i8; 2]) -> Vertex {
        Vertex {
            pos: [p[0] as f32, p[1] as f32, p[2] as f32, 1.0],
            tex_coord: [t[0] as f32, t[1] as f32],
        }
    }
}

struct App<R: gfx::Resources>{
    bundle: Bundle<R, pipe::Data<R>>,
    texture: Texture<R, gfx::format::R8_G8_B8_A8>,
    data: Box<[[[u8; 4]; 1000]; 1000]>,
    invalid: bool
}

const PALETTE: [[u8; 4]; 16] = [
    [255, 255, 255, 255],
    [228, 228, 228, 255],
    [136, 136, 136, 255],
    [ 34,  34,  34, 255],
    [255, 167, 209, 255],
    [229,   0,   0, 255],
    [229, 149,   0, 255],
    [160, 106,  66, 255],
    [229, 217,   0, 255],
    [148, 224,  68, 255],
    [  2, 190,   1, 255],
    [  0, 211, 221, 255],
    [  0, 131, 199, 255],
    [  0,   0, 234, 255],
    [207, 110, 228, 255],
    [130,   0, 128, 255],
];

impl<R: gfx::Resources> gfx_app::Application<R> for App<R> {
    fn new<F: gfx::Factory<R>>(factory: &mut F, backend: gfx_app::shade::Backend, window_targets: gfx_app::WindowTargets<R>) -> Self {
        use gfx::traits::FactoryExt;

        let vs = gfx_app::shade::Source {
            glsl_120: include_bytes!("shader/simple.glslv"),
            glsl_150: include_bytes!("shader/simple.glslv"),
            glsl_es_100: include_bytes!("shader/simple.glslv"),
            .. gfx_app::shade::Source::empty()
        };
        let ps = gfx_app::shade::Source {
            glsl_120: include_bytes!("shader/simple.glslf"),
            glsl_150: include_bytes!("shader/simple.glslf"),
            glsl_es_100: include_bytes!("shader/simple.glslf"),
            .. gfx_app::shade::Source::empty()
        };

        let vertex_data = [
            Vertex::new([-1, -1,  0], [0, 0]),
            Vertex::new([ 1, -1,  0], [1, 0]),
            Vertex::new([ 1,  1,  0], [1, 1]),
            Vertex::new([-1,  1,  0], [0, 1]),
        ];

        let index_data: &[u16] = &[
             0,  1,  2,  2,  3,  0,
        ];

        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, index_data);

        let texture = factory.create_texture(
            texture::Kind::D2(1000, 1000, texture::AaMode::Single),
            1,
            SHADER_RESOURCE | TRANSFER_DST | TRANSFER_SRC,
            Usage::Dynamic,
            None
        ).unwrap();

        let texture_view = factory.view_texture_as_shader_resource::<gfx::format::Srgba8>(
            &texture,
            (0, 0),
            Swizzle::new()
        ).unwrap();

        let mut buffer: Box<[[[u8; 4]; 1000]; 1000]> = unsafe {
            let mut buffer: Vec<[u8; 4]> = vec![[255, 255, 0, 255]; 1_000_000];
            let result = buffer.as_mut_ptr();
            mem::forget(buffer);
            Box::from_raw(mem::transmute(result))
        };

        {
            let mut file = reqwest::get("https://www.reddit.com/api/place/board-bitmap").unwrap();
            let mut tmp = [0u8; 500];
            file.read_exact(&mut tmp[0..4]).unwrap();
            for y in 0..1000 {
                let target = &mut (*buffer)[999-y];
                file.read_exact(&mut tmp).unwrap();
                for x in 0..500 {
                    let index0 = (tmp[x] & 0xF) as usize;
                    let index1 = (tmp[x] >> 4) as usize;
                    target[x*2] = PALETTE[index1];
                    target[x*2+1] = PALETTE[index0];
                }
            }
        }


        // let (texture, texture_view) = factory.create_texture_immutable_u8::<gfx::format::Srgba8>(
        //     texture::Kind::D2(1000, 1000, texture::AaMode::Single),
        //     &[unsafe { mem::transmute::<&[[[u8; 4]; 1000]; 1000], &[u8; 1_000_000]>(&*buffer) }]
        // ).unwrap();

        let sinfo = texture::SamplerInfo::new(
            texture::FilterMethod::Scale,
            texture::WrapMode::Clamp
        );

        let pso = factory.create_pipeline_simple(
            vs.select(backend).unwrap(),
            ps.select(backend).unwrap(),
            pipe::new()
        ).unwrap();

        let data = pipe::Data {
            vbuf: vbuf,
            color: (texture_view, factory.create_sampler(sinfo)),
            out_color: window_targets.color,
        };

        App {
            bundle: Bundle::new(slice, pso, data),
            texture: texture,
            data: buffer,
            invalid: true
        }
    }

    fn render<C: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C>) {

        // {
        //     let mut file = reqwest::get("https://www.reddit.com/api/place/board-bitmap").unwrap();
        //     let mut tmp = [0u8; 500];
        //     file.read_exact(&mut tmp[0..4]).unwrap();
        //     for y in 0..1000 {
        //         let target = &mut (*self.data)[999-y];
        //         file.read_exact(&mut tmp).unwrap();
        //         for x in 0..500 {
        //             let index0 = (tmp[x] & 0xF) as usize;
        //             let index1 = (tmp[x] >> 4) as usize;
        //             target[x*2] = PALETTE[index1];
        //             target[x*2+1] = PALETTE[index0];
        //         }
        //     }
        // }

        encoder.update_texture::<_, gfx::format::Srgba8>(&self.texture, None, self.texture.get_info().to_image_info(0), unsafe {
            mem::transmute::<&[[[u8; 4]; 1000]; 1000], &[[u8; 4]; 1_000_000]>(&*self.data)
        }).unwrap();

        encoder.clear(&self.bundle.data.out_color, [0.1, 0.2, 0.3, 1.0]);
        self.bundle.encode(encoder);
    }

    fn on_resize(&mut self, window_targets: gfx_app::WindowTargets<R>) {
        self.bundle.data.out_color = window_targets.color;
    }
}

fn main() {
    use gfx_app::{launch_gl3, Wrap};
    let wb = winit::WindowBuilder::new().with_title("ruplace");
    launch_gl3::<Wrap<_, _, App<_>>>(wb);
}
