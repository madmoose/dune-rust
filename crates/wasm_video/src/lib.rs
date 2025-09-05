#![allow(clippy::identity_op)]

use std::{cell::RefCell, rc::Rc};

use dune::{Framebuffer, Palette, hnm};
use wasm_bindgen::{Clamped, prelude::*};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, console};

static HNM: &[u8] = include_bytes!("../../../assets/CRYO.HNM");

#[wasm_bindgen]
struct HnmVideoRenderer {
    _inner: Rc<RefCell<HnmVideoRendererInner>>,
}

#[allow(unused)]
#[wasm_bindgen]
impl HnmVideoRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> HnmVideoRenderer {
        HnmVideoRenderer {
            _inner: HnmVideoRendererInner::new(canvas),
        }
    }
}

struct HnmVideoRendererInner {
    renderer: hnm::HnmDecoder<'static>,
    canvas: HtmlCanvasElement,
    pal: Palette,
    framebuffer: Framebuffer,
    image: Vec<u8>,
    last_frame_time: Option<f64>,
    frame: usize,
}

impl HnmVideoRendererInner {
    pub fn new(canvas: HtmlCanvasElement) -> Rc<RefCell<HnmVideoRendererInner>> {
        let mut pal = Palette::new();
        let framebuffer = Framebuffer::new(320, 200);
        let image = vec![0; 4 * 320 * 200];
        let renderer = hnm::HnmDecoder::new(HNM, &mut pal).unwrap();

        let r = Rc::new(RefCell::new(HnmVideoRendererInner {
            renderer,
            canvas,
            image,
            pal,
            framebuffer,
            last_frame_time: None,
            frame: 0,
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

    pub fn animate(&mut self, _frame_delta: f32) {
        self.frame += 1;
    }

    pub fn draw(&mut self) -> Result<(), JsValue> {
        if let Err(err) =
            self.renderer
                .decode_frame(self.frame, &mut self.framebuffer, &mut self.pal)
        {
            console::error_1(&format!("{err:?}").into());
            return Ok(());
        }

        let context_options = serde_wasm_bindgen::to_value(&serde_json::json!({
            "premultipliedAlpha": false,
            "alpha": false,
        }))
        .inspect_err(|err| console::error_1(&format!("{err:?}").into()))
        .unwrap();

        let context = self
            .canvas
            .get_context_with_context_options("2d", &context_options)?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        if false {
            for y in 0..200 {
                for x in 0..320 {
                    let c = if y % 2 == 0 && x % 2 == 0 {
                        self.framebuffer.get(x / 2, y / 2)
                    } else {
                        0
                    };
                    let rgb = self.pal.get_rgb888(c as usize);

                    let ofs = 4 * (y * 320 + x) as usize;
                    self.image[ofs + 0] = rgb.0;
                    self.image[ofs + 1] = rgb.1;
                    self.image[ofs + 2] = rgb.2;
                    self.image[ofs + 3] = 255;
                }
            }
        } else {
            for y in 0..200 {
                for x in 0..320 {
                    let c = self.framebuffer.get(x, y);
                    let rgb = self.pal.get_rgb888(c as usize);
                    let ofs = 4 * (y * 320 + x) as usize;

                    self.image[ofs + 0] = rgb.0;
                    self.image[ofs + 1] = rgb.1;
                    self.image[ofs + 2] = rgb.2;
                    self.image[ofs + 3] = 255;
                }
            }
        }
        let Ok(image_data) =
            ImageData::new_with_u8_clamped_array_and_sh(Clamped(self.image.as_slice()), 320, 200)
                .inspect_err(|err| console::error_1(&format!("{err:?}").into()))
        else {
            return Ok(());
        };

        context
            .put_image_data(&image_data, 0.0, 0.0)
            .inspect_err(|err| console::error_1(&format!("{err:?}").into()))
            .unwrap();

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
