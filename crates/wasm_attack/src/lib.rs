#![allow(clippy::identity_op)]

use std::{cell::RefCell, rc::Rc};

use dune::{Framebuffer, Palette, attack::AttackState};
use wasm_bindgen::{Clamped, prelude::*};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[wasm_bindgen]
struct AttackRenderer {
    inner: Rc<RefCell<AttackRendererInner>>,
}

#[allow(unused)]
#[wasm_bindgen]
impl AttackRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> AttackRenderer {
        AttackRenderer {
            inner: AttackRendererInner::new(canvas),
        }
    }

    pub fn set_rand_bits(&mut self, seed: u16) {
        self.inner.borrow_mut().attack.set_rand_bits(seed);
    }

    pub fn set_rng_seed(&mut self, seed: u16) {
        self.inner.borrow_mut().attack.set_rng_seed(seed);
    }

    pub fn set_masked_rng_seed(&mut self, seed: u16) {
        self.inner.borrow_mut().attack.set_masked_rng_seed(seed);
    }
}

struct AttackRendererInner {
    attack: AttackState,
    canvas: HtmlCanvasElement,
    image: Vec<u8>,
    last_frame_time: Option<f64>,
    _frame: usize,
    accumulated_time: f32,
}

impl AttackRendererInner {
    pub fn new(canvas: HtmlCanvasElement) -> Rc<RefCell<AttackRendererInner>> {
        let image = vec![0; 4 * 320 * 200];

        let attack = AttackState::default();

        let r = Rc::new(RefCell::new(AttackRendererInner {
            attack,
            canvas,
            image,
            last_frame_time: None,
            _frame: 0,
            accumulated_time: 0.0,
        }));

        let f = Rc::new(RefCell::new(None));
        let g = f.clone();
        let r2 = r.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            let mut r = r2.borrow_mut();

            let now = now();
            if let Some(last_frame_time) = r.last_frame_time {
                let frame_delta = (now - last_frame_time) as f32;

                r.step_frame(frame_delta);
            }
            r.last_frame_time = Some(now);

            r.draw().unwrap();

            request_animation_frame(f.borrow().as_ref().unwrap());
        }));
        request_animation_frame(g.borrow().as_ref().unwrap());

        r
    }

    fn step_frame(&mut self, frame_delta: f32) {
        const FRAME_TIME: f32 = 1.0 / 30.0;

        self.accumulated_time += frame_delta / 1000.0; // Convert milliseconds to seconds

        while self.accumulated_time >= FRAME_TIME {
            self.attack.step_frame();
            self.accumulated_time -= FRAME_TIME;
        }
    }

    fn draw(&mut self) -> Result<(), JsValue> {
        let mut pal = Palette::new();
        let mut frame = Framebuffer::new(320, 200);
        frame.clear();

        self.attack.draw(&mut frame, &mut pal);

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
                let rgb = pal.get_rgb888(c as usize);

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
