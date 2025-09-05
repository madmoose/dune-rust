#![allow(clippy::identity_op)]

use std::{cell::RefCell, rc::Rc};

use dune::{Framebuffer, IndexMap, Palette, SpriteSheet};
use room_renderer::{DrawOptions, Room, RoomSheet};
use serde::Deserialize;
use wasm_bindgen::{Clamped, prelude::*};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, console};

static ROOMS_SIET: &[u8] = include_bytes!("../../../assets/SIET.SAL");
static ROOMS_PALACE: &[u8] = include_bytes!("../../../assets/PALACE.SAL");
static ROOMS_VILG: &[u8] = include_bytes!("../../../assets/VILG.SAL");
static ROOMS_HARK: &[u8] = include_bytes!("../../../assets/HARK.SAL");

static SPRITE_SHEET_PROUGE: &[u8] = include_bytes!("../../../assets/PROUGE.BIN");
static SPRITE_SHEET_COMM: &[u8] = include_bytes!("../../../assets/COMM.BIN");
static SPRITE_SHEET_EQUI: &[u8] = include_bytes!("../../../assets/EQUI.BIN");
static SPRITE_SHEET_BALCON: &[u8] = include_bytes!("../../../assets/BALCON.BIN");
static SPRITE_SHEET_CORR: &[u8] = include_bytes!("../../../assets/CORR.BIN");
static SPRITE_SHEET_POR: &[u8] = include_bytes!("../../../assets/POR.BIN");
static SPRITE_SHEET_SIET1: &[u8] = include_bytes!("../../../assets/SIET1.BIN");
static SPRITE_SHEET_XPLAIN9: &[u8] = include_bytes!("../../../assets/XPLAIN9.BIN");
static SPRITE_SHEET_BUNK: &[u8] = include_bytes!("../../../assets/BUNK.BIN");
static SPRITE_SHEET_SERRE: &[u8] = include_bytes!("../../../assets/SERRE.BIN");
static SPRITE_SHEET_BOTA: &[u8] = include_bytes!("../../../assets/BOTA.BIN");

static _SKY: &[u8] = include_bytes!("../../../assets/SKY.BIN");
static SKYDN: &[u8] = include_bytes!("../../../assets/SKYDN.BIN");

#[wasm_bindgen]
struct RoomRenderer {
    inner: Rc<RefCell<RoomRendererInner>>,
}

#[derive(Debug, Deserialize)]
struct RoomRendererDrawOptions {
    draw_sprites: bool,
    draw_polygons: bool,
    draw_lines: bool,
    highlighted_index: Option<usize>,
    sky_palette: Option<usize>,
}

#[allow(unused)]
#[wasm_bindgen]
impl RoomRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> RoomRenderer {
        RoomRenderer {
            inner: RoomRendererInner::new(canvas),
        }
    }

    pub fn set_room_sheet(&mut self, room_sheet_name: &str) {
        if let Some(room_sheet) = room_sheet_by_name(room_sheet_name) {
            self.inner.borrow_mut().set_room_sheet(room_sheet);
        }
    }

    pub fn get_room_count(&self) -> usize {
        self.inner.borrow().get_room_count()
    }

    pub fn set_room_index(&mut self, index: usize) {
        self.inner.borrow_mut().set_room_index(index);
    }

    pub fn set_sprite_sheet(&mut self, sprite_sheet_name: &str) {
        if let Some(sprite_sheet) = sprite_sheet_by_name(sprite_sheet_name) {
            self.inner.borrow_mut().set_sprite_sheet(sprite_sheet);
        }
    }

    pub fn get_index_of_part_at_position(&self, x: i16, y: i16) -> Option<usize> {
        self.inner.borrow_mut().get_index_of_part_at_position(x, y)
    }

    pub fn draw(&mut self, options: JsValue) -> Result<(), JsValue> {
        let options: RoomRendererDrawOptions = serde_wasm_bindgen::from_value(options).unwrap();
        self.inner.borrow_mut().draw(options)
    }

    pub fn get_room(&self) -> wasm_bindgen::JsValue {
        let inner = self.inner.borrow();
        let room = inner.get_room();

        serde_wasm_bindgen::to_value(&room).unwrap()
    }
}

struct RoomRendererInner {
    room_renderer: room_renderer::RoomRenderer,
    room_sheet: Option<RoomSheet>,
    room_index: usize,
    canvas: HtmlCanvasElement,
    image: Vec<u8>,
    index_map: Option<IndexMap>,
}

impl RoomRendererInner {
    pub fn new(canvas: HtmlCanvasElement) -> Rc<RefCell<RoomRendererInner>> {
        let room_renderer = room_renderer::RoomRenderer::new();

        let image = vec![0; 4 * 320 * 200];
        let index_map = Some(IndexMap::new());

        Rc::new(RefCell::new(RoomRendererInner {
            room_renderer,
            room_sheet: None,
            room_index: 0,
            canvas,
            image,
            index_map,
        }))
    }

    fn set_room_sheet(&mut self, room_sheet: RoomSheet) {
        self.room_sheet = Some(room_sheet);

        // Ensure that we reload the room when changing room sheet
        self.set_room_index(0);
    }

    fn get_room_count(&self) -> usize {
        self.room_sheet
            .as_ref()
            .map(|room_sheet| room_sheet.room_count())
            .unwrap_or_default()
    }

    fn set_room_index(&mut self, index: usize) {
        self.room_index = index;
        let Some(room) = self
            .room_sheet
            .as_ref()
            .and_then(|room_sheet| room_sheet.get_room(index))
        else {
            return;
        };
        self.room_renderer.set_room(room.clone());
    }

    fn set_sprite_sheet(&mut self, sprite_sheet: SpriteSheet) {
        self.room_renderer.set_sprite_sheet(sprite_sheet);
    }

    fn get_index_of_part_at_position(&self, x: i16, y: i16) -> Option<usize> {
        let x = x.try_into().ok()?;
        let y = y.try_into().ok()?;
        self.index_map.as_ref().and_then(|m| m.get_index(x, y))
    }

    fn draw(&mut self, options: RoomRendererDrawOptions) -> Result<(), JsValue> {
        let mut pal = Palette::new();
        let mut frame = Framebuffer::new(320, 200);
        frame.clear();

        if let Some(index_map) = self.index_map.as_mut() {
            index_map.clear();
        }

        if let Some(sky_palette_index) = options.sky_palette {
            self.room_renderer
                .draw_sky(SKYDN, sky_palette_index, &mut pal);
        }

        let Some(sprite_sheet) = self.room_renderer.get_sprite_sheet() else {
            return Ok(());
        };

        sprite_sheet.apply_palette_update(&mut pal).unwrap();

        let res = self.room_renderer.draw(
            &DrawOptions {
                draw_sprites: options.draw_sprites,
                draw_polygons: options.draw_polygons,
                draw_lines: options.draw_lines,
            },
            &mut frame,
            self.index_map.as_mut(),
        );
        if let Err(error) = res {
            console::error_1(&format!("{error:#?}").into());
            return Ok(());
        }

        let context_options = serde_wasm_bindgen::to_value(&serde_json::json!({
            "premultipliedAlpha": false,
            "alpha": false,
        }))
        .unwrap();

        let context = self
            .canvas
            .get_context_with_context_options("2d", &context_options)?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        for y in 0..200 {
            for x in 0..320 {
                let c = frame.get(x, y);
                let mut rgb = pal.get_rgb888(c as usize);

                if let Some(highlighted_index) = options.highlighted_index
                    && self.index_map.as_ref().is_some_and(|index_map| {
                        index_map.get_index(x, y) == Some(highlighted_index)
                    })
                {
                    rgb.0 = (256 - ((256 - rgb.0 as usize) / 2)) as u8;
                    rgb.1 = (256 - ((256 - rgb.1 as usize) / 2)) as u8;
                    rgb.2 = (256 - ((256 - rgb.2 as usize) / 2)) as u8;
                }

                let ofs = 4 * (y * 320 + x) as usize;
                self.image[ofs + 0] = rgb.0;
                self.image[ofs + 1] = rgb.1;
                self.image[ofs + 2] = rgb.2;
                self.image[ofs + 3] = 255;
            }
        }

        let image_data =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(self.image.as_slice()), 320, 200)
                .expect("Failed to create ImageData");

        context
            .put_image_data(&image_data, 0.0, 0.0)
            .expect("put_image_data failed");

        Ok(())
    }

    pub fn get_room(&self) -> Option<&Room> {
        self.room_sheet
            .as_ref()
            .and_then(|room_sheet| room_sheet.get_room(self.room_index))
    }
}

fn room_sheet_by_name(room_sheet: &str) -> Option<room_renderer::RoomSheet> {
    let room_sheet_data = match room_sheet {
        "SIET" => ROOMS_SIET,
        "PALACE" => ROOMS_PALACE,
        "VILG" => ROOMS_VILG,
        "HARK" => ROOMS_HARK,
        _ => ROOMS_PALACE,
    };

    room_renderer::RoomSheet::new(room_sheet_data).ok()
}

fn sprite_sheet_by_name(sprite_sheet: &str) -> Option<SpriteSheet> {
    let sprite_sheet_data = match sprite_sheet {
        "PROUGE" => SPRITE_SHEET_PROUGE,
        "COMM" => SPRITE_SHEET_COMM,
        "EQUI" => SPRITE_SHEET_EQUI,
        "BALCON" => SPRITE_SHEET_BALCON,
        "CORR" => SPRITE_SHEET_CORR,
        "POR" => SPRITE_SHEET_POR,
        "SIET1" => SPRITE_SHEET_SIET1,
        "XPLAIN9" => SPRITE_SHEET_XPLAIN9,
        "BUNK" => SPRITE_SHEET_BUNK,
        "SERRE" => SPRITE_SHEET_SERRE,
        "BOTA" => SPRITE_SHEET_BOTA,
        _ => SPRITE_SHEET_POR,
    };

    SpriteSheet::from_slice(sprite_sheet_data).ok()
}
