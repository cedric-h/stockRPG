use crate::prelude::*;
use nuklear_backend_wgpurs::Drawer;
use nulkear::{
    Allocator, FontConfig, FontAtlas, Handle,
};

pub struct NuklearState {
    allo: Allocator,
    font_cfg: FontCon
}

impl NuklearState {
    fn new() -> Self {
        let allo = Allocator::new_vec();
    }

    fn allocate_media(&mut self, drawer: &Drawer) {
    }
}

pub struct Media {
    font_atlas: FontAtlas,
    font_14: FontID,
    font_18: FontID,
    font_20: FontID,
    font_22: FontID,

    font_tex: Handle,
}

impl Media {
    pub fn new(nk_state: &NuklearState, drawer: &Drawer) -> Self {
        let mut cfg = FontConfig::with_size(0.0);
        cfg.set_oversample_h(3);
        cfg.set_oversample_v(2);
        cfg.set_glyph_range(font_cyrillic_glyph_ranges());
        cfg.set_ttf(include_bytes!("./font/SDS_8x8.ttf"));

        let mut atlas = FontAtlas::new(&mut nk_state.allo);

        cfg.set_ttf_data_owned_by_atlas(false);
        cfg.set_size(14_f32);
        let font_14 = atlas.add_font_with_config(&cfg).unwrap();

        cfg.set_ttf_data_owned_by_atlas(false);
        cfg.set_size(18_f32);
        let font_18 = atlas.add_font_with_config(&cfg).unwrap();

        cfg.set_ttf_data_owned_by_atlas(false);
        cfg.set_size(20_f32);
        let font_20 = atlas.add_font_with_config(&cfg).unwrap();

        cfg.set_ttf_data_owned_by_atlas(false);
        cfg.set_size(22_f32);
        let font_22 = atlas.add_font_with_config(&cfg).unwrap();

        let font_tex = {
            let (b, w, h) = atlas.bake(FontAtlasFormat::Rgba32);
            drawer.add_texture(&mut device, b, w, h)
        };

        let mut null = DrawNullTexture::default();

        atlas.end(font_tex, Some(&mut null));

        Self {
            font_atlas: atlas,
            font_14,
            font_18,
            font_20,
            font_22,
            font_tex,
        }
    }
}
