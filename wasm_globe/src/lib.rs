#![allow(clippy::identity_op)]

use std::{cell::RefCell, rc::Rc};

use framebuffer::Framebuffer;
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

static MAP: &[u8] = include_bytes!("../../assets/MAP.BIN");
static GLOBDATA: &[u8] = include_bytes!("../../assets/GLOBDATA.BIN");
static TABLAT: &[u8] = include_bytes!("../../assets/TABLAT.BIN");
static PAL: &[u8] = include_bytes!("../../assets/PAL.BIN");
static FRESK: &[u8] = include_bytes!("../../assets/FRESK.BIN");
static ICONES: &[u8] = include_bytes!("../../assets/ICONES.BIN");

#[wasm_bindgen]
struct GlobeRenderer {
    inner: Rc<RefCell<GlobeRendererInner>>,
}

#[allow(unused)]
#[wasm_bindgen]
impl GlobeRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> GlobeRenderer {
        GlobeRenderer {
            inner: GlobeRendererInner::new(canvas),
        }
    }

    #[wasm_bindgen(method, getter)]
    pub fn rotation(&self) -> u16 {
        self.inner.as_ref().borrow().rotation as u16
    }

    #[wasm_bindgen(method, setter)]
    pub fn set_rotation(&mut self, value: u16) {
        self.inner.borrow_mut().rotation = value as f32;
    }

    #[wasm_bindgen(method, getter)]
    pub fn tilt(&self) -> i16 {
        self.inner.as_ref().borrow().tilt as i16
    }

    #[wasm_bindgen(method, setter)]
    pub fn set_tilt(&mut self, value: i16) {
        self.inner.borrow_mut().tilt = value as f32;
    }

    pub fn click(&mut self, x: u16, y: u16) -> Result<(), JsValue> {
        self.inner.borrow_mut().click(x, y)
    }
}

struct GlobeRendererInner {
    renderer: globe_renderer::GlobeRenderer,
    canvas: HtmlCanvasElement,
    buffer: Vec<u8>,
    image: Vec<u8>,
    rotation: f32,
    tilt: f32,
    rotation_target: f32,
    tilt_target: f32,
    last_frame_time: Option<f64>,
    animating: bool,
}

struct UIIcon {
    pub sprite_index: u16,
    pub x: usize,
    pub y: usize,
}

fn smallest(a: f32, b: f32) -> f32 {
    if a.abs() < b.abs() {
        a
    } else {
        b
    }
}

impl GlobeRendererInner {
    pub fn new(canvas: HtmlCanvasElement) -> Rc<RefCell<GlobeRendererInner>> {
        let renderer = globe_renderer::GlobeRenderer::new(GLOBDATA, MAP, TABLAT);
        let buffer = vec![0; 320 * 200];
        let image = vec![0; 4 * 320 * 200];

        let r = Rc::new(RefCell::new(GlobeRendererInner {
            renderer,
            canvas,
            buffer,
            image,
            rotation: 0.0,
            tilt: 0.0,
            rotation_target: 0.0,
            tilt_target: 0.0,
            last_frame_time: None,
            animating: false,
        }));

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        let r2 = r.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            let mut r = r2.borrow_mut();

            let now = now();
            if let Some(last_frame_time) = r.last_frame_time {
                let frame_delta = (now - last_frame_time) as f32;

                r.animate(frame_delta);
            }
            r.last_frame_time = Some(now);

            r.draw().unwrap();

            request_animation_frame(f.borrow().as_ref().unwrap());
        }));
        request_animation_frame(g.borrow().as_ref().unwrap());

        r
    }

    fn draw_background(fb: &mut Framebuffer) {
        sprite::draw_sprite_from_sprite_sheet(fb, FRESK, 0, 0, 0).unwrap();
        sprite::draw_sprite_from_sprite_sheet(fb, FRESK, 1, 214, 0).unwrap();
        sprite::draw_sprite_from_sprite_sheet(fb, FRESK, 2, 91, 20).unwrap();

        #[rustfmt::skip]
        let icons = [
            UIIcon { sprite_index:  6, x:   0, y: 152, },
            UIIcon { sprite_index:  3, x: 228, y: 152, },
            UIIcon { sprite_index: 13, x:  22, y: 161, },
            UIIcon { sprite_index: 14, x:  92, y: 152, },
            UIIcon { sprite_index: 12, x:   2, y: 154, },
            UIIcon { sprite_index: 12, x: 317, y: 154, },
            UIIcon { sprite_index: 27, x:  92, y: 159, },
            UIIcon { sprite_index: 27, x:  92, y: 167, },
            UIIcon { sprite_index: 27, x:  92, y: 175, },
            UIIcon { sprite_index: 27, x:  92, y: 183, },
            UIIcon { sprite_index: 27, x:  92, y: 191, },
            UIIcon { sprite_index: 41, x: 266, y: 171, },
            UIIcon { sprite_index: 49, x:  38, y: 159, },
            UIIcon { sprite_index: 50, x:  54, y: 168, },
            UIIcon { sprite_index: 51, x:  38, y: 183, },
            UIIcon { sprite_index: 52, x:  20, y: 168, },
            UIIcon { sprite_index: 53, x:  36, y: 172, },
        ];

        for icon in icons {
            sprite::draw_sprite_from_sprite_sheet(fb, ICONES, icon.sprite_index, icon.x, icon.y)
                .unwrap();
        }
    }

    fn draw_head(fb: &mut Framebuffer, head: u8) {
        let head = (16 + head.clamp(0, 10)) as u16;
        sprite::draw_sprite_from_sprite_sheet(fb, ICONES, 15, 126, 148).unwrap();
        sprite::draw_sprite_from_sprite_sheet(fb, ICONES, head, 150, 137).unwrap();
    }

    pub fn animate(&mut self, frame_delta: f32) {
        if !self.animating {
            self.rotation = f32::rem_euclid(self.rotation + 0.1 * frame_delta, 65536.0);
            return;
        }

        let mut rotation_delta = self.rotation_target - self.rotation;
        let tilt_delta = self.tilt_target - self.tilt;

        if 65536.0 - rotation_delta.abs() < rotation_delta.abs() {
            rotation_delta = 65536.0 - rotation_delta.abs();
        }

        let mut done_rotating = false;
        let mut done_tilting = false;

        if rotation_delta.abs() < 100.0 {
            self.rotation = self.rotation_target;
            done_rotating = true;
        } else {
            self.rotation += smallest(rotation_delta, rotation_delta.signum() * 50.0 * frame_delta);
        }

        if tilt_delta.abs() < 1.0 {
            self.tilt = self.tilt_target;
            done_tilting = true;
        } else {
            self.tilt += smallest(tilt_delta, tilt_delta.signum() * 0.2 * frame_delta);
        }

        self.animating = !done_rotating || !done_tilting;
    }

    pub fn draw(&mut self) -> Result<(), JsValue> {
        let mut framebuffer = Framebuffer::new_with_pixel_data(320, 200, &mut self.buffer);
        for i in 0..256 {
            let r = ((PAL[3 * i + 0] as u32) * 63 / 255) as u8;
            let g = ((PAL[3 * i + 1] as u32) * 63 / 255) as u8;
            let b = ((PAL[3 * i + 2] as u32) * 63 / 255) as u8;
            framebuffer.mut_pal().set(i, (r, g, b));
        }

        framebuffer.clear();

        Self::draw_background(&mut framebuffer);
        Self::draw_head(&mut framebuffer, 10);
        self.renderer
            .draw(&mut framebuffer, self.rotation as u16, self.tilt as i16);

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
                let rgb = framebuffer.get_rgb(x, y);

                self.image[4 * (y * 320 + x) + 0] = rgb.0;
                self.image[4 * (y * 320 + x) + 1] = rgb.1;
                self.image[4 * (y * 320 + x) + 2] = rgb.2;
                self.image[4 * (y * 320 + x) + 3] = 255;
            }
        }

        let image_data =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(self.image.as_slice()), 320, 200)
                .expect("Failed to create ImageData");

        context.put_image_data(&image_data, 0.0, 0.0).unwrap();

        Ok(())
    }

    pub fn click(&mut self, x: u16, y: u16) -> Result<(), JsValue> {
        if !self.animating {
            if x >= 38 && y >= 159 && x < 54 && y < 172 {
                self.tilt -= 8.0;
            }
            if x >= 54 && y >= 168 && x < 72 && y < 185 {
                self.rotation = f32::rem_euclid(self.rotation - 4096.0, 65536.0);
            }
            if x >= 38 && y >= 183 && x < 54 && y < 199 {
                self.tilt += 8.0;
            }
            if x >= 20 && y >= 168 && x < 37 && y < 185 {
                self.rotation = f32::rem_euclid(self.rotation + 4096.0, 65536.0);
            }
        }
        if x >= 36 && y >= 172 && x < 57 && y < 182 {
            self.tilt_target = 0.0;
            self.rotation_target = 0.0;
            self.animating = true;
        }

        Ok(())
    }
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn now() -> f64 {
    window()
        .performance()
        .expect("no `performance` object on `window`")
        .now()
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
