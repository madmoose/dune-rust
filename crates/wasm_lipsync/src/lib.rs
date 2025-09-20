#![allow(clippy::identity_op)]

use core::f32;
use std::{cell::RefCell, rc::Rc};

use dune::{Framebuffer, Lipsync, Palette, SpriteSheet};
use serde::Serialize;
use wasm_bindgen::{Clamped, prelude::*};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, console};

static LETO: &[u8] = include_bytes!("../../../assets/LETO.BIN");
static JESS: &[u8] = include_bytes!("../../../assets/JESS.BIN");
static HAWA: &[u8] = include_bytes!("../../../assets/HAWA.BIN");
static IDAH: &[u8] = include_bytes!("../../../assets/IDAH.BIN");
static GURN: &[u8] = include_bytes!("../../../assets/GURN.BIN");
static STIL: &[u8] = include_bytes!("../../../assets/STIL.BIN");
static KYNE: &[u8] = include_bytes!("../../../assets/KYNE.BIN");
static CHAN: &[u8] = include_bytes!("../../../assets/CHAN.BIN");
static HARA: &[u8] = include_bytes!("../../../assets/HARA.BIN");
static BARO: &[u8] = include_bytes!("../../../assets/BARO.BIN");
static FEYD: &[u8] = include_bytes!("../../../assets/FEYD.BIN");
static EMPR: &[u8] = include_bytes!("../../../assets/EMPR.BIN");
static HARK: &[u8] = include_bytes!("../../../assets/HARK.BIN");
static SMUG: &[u8] = include_bytes!("../../../assets/SMUG.BIN");
static FRM1: &[u8] = include_bytes!("../../../assets/FRM1.BIN");
static FRM2: &[u8] = include_bytes!("../../../assets/FRM2.BIN");
static FRM3: &[u8] = include_bytes!("../../../assets/FRM3.BIN");

// static PA001O_VOC: &[u8] = include_bytes!("../../../assets/PA001O.VOC");

static RESOURCES: [&[u8]; 17] = [
    LETO, JESS, HAWA, IDAH, GURN, STIL, KYNE, CHAN, HARA, BARO, FEYD, EMPR, HARK, SMUG, FRM1, FRM2,
    FRM3,
];

#[wasm_bindgen]
struct PortraitRenderer {
    _inner: Rc<RefCell<PortraitRendererInner>>,
}

#[allow(unused)]
#[wasm_bindgen]
impl PortraitRenderer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> PortraitRenderer {
        PortraitRenderer {
            _inner: PortraitRendererInner::new(canvas),
        }
    }

    pub fn get_media_information(&self) -> JsValue {
        let names = [
            "LETO", "JESS", "HAWA", "IDAH", "GURN", "STIL", "KYNE", "CHAN", "HARA", "BARO", "FEYD",
            "EMPR", "HARK", "SMUG", "FRM1", "FRM2", "FRM3",
        ];

        let mut portraits = Vec::with_capacity(RESOURCES.len());

        for (i, &resource) in RESOURCES.iter().enumerate() {
            let sprite_sheet = SpriteSheet::from_slice(resource).unwrap();
            let last_resource_id = sprite_sheet.resource_count() - 1;
            let lipsync_data = sprite_sheet.get_resource(last_resource_id).unwrap();
            let lipsync = Lipsync::from_bytes(lipsync_data);

            let name = names.get(i).map(|s| s.to_string()).unwrap_or_default();
            let animation_count = lipsync.animations.len();

            portraits.push(PortraitInformation {
                name,
                animation_count,
            });
        }

        serde_wasm_bindgen::to_value(&portraits).unwrap()
    }

    pub fn play_portrait_animation(&mut self, portrait_index: usize, animation_index: usize) {
        self._inner.borrow_mut().playback_state = PlaybackState::Switch {
            portrait_index,
            animation_index,
        };
    }
}

#[derive(Serialize)]
pub struct PortraitInformation {
    name: String,
    animation_count: usize,
}

#[derive(PartialEq)]
enum PlaybackState {
    Stopped,
    Playing,
    Switch {
        portrait_index: usize,
        animation_index: usize,
    },
}

struct PortraitRendererInner {
    canvas: HtmlCanvasElement,

    pal: Palette,
    framebuffer: Framebuffer,
    image: Vec<u8>,

    playback_state: PlaybackState,
    last_frame_time: Option<f64>,

    sprite_sheet: SpriteSheet,
    lipsync: Lipsync,
    portrait_index: usize,
    animation_index: usize,
    frame_index: usize,
}

impl PortraitRendererInner {
    pub fn new(canvas: HtmlCanvasElement) -> Rc<RefCell<PortraitRendererInner>> {
        let mut pal = Palette::new();
        let framebuffer = Framebuffer::new(320, 200);
        let image = vec![0; 4 * 320 * 200];
        let portrait_index = 0;
        let sprite_sheet = SpriteSheet::from_slice(RESOURCES[portrait_index]).unwrap();
        let last_resource_id = sprite_sheet.resource_count() - 1;
        let lipsync_data = sprite_sheet.get_resource(last_resource_id).unwrap();
        let lipsync = Lipsync::from_bytes(lipsync_data);

        let _ = sprite_sheet.apply_palette_update(&mut pal);

        let r = Rc::new(RefCell::new(PortraitRendererInner {
            canvas,
            image,
            pal,
            framebuffer,
            playback_state: PlaybackState::Playing,
            last_frame_time: None,
            sprite_sheet,
            lipsync,
            portrait_index,
            animation_index: 0,
            frame_index: 0,
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
        self.frame_index += 1;
        self.framebuffer.clear();
    }

    pub fn draw(&mut self) -> Result<(), JsValue> {
        match self.playback_state {
            PlaybackState::Stopped => {
                return Ok(());
            }
            PlaybackState::Playing => {}
            PlaybackState::Switch {
                portrait_index,
                animation_index,
            } => {
                self.portrait_index = portrait_index;
                self.animation_index = animation_index;
                self.frame_index = 0;

                let sprite_sheet = SpriteSheet::from_slice(RESOURCES[self.portrait_index]).unwrap();
                let last_resource_id = sprite_sheet.resource_count() - 1;
                let lipsync_data = sprite_sheet.get_resource(last_resource_id).unwrap();
                let lipsync = Lipsync::from_bytes(lipsync_data);

                let _ = sprite_sheet.apply_palette_update(&mut self.pal);

                self.sprite_sheet = sprite_sheet;
                self.lipsync = lipsync;

                self.framebuffer.clear();
                self.playback_state = PlaybackState::Playing;
            }
        }

        if self.animation_index >= self.lipsync.animation_count() {
            console::error_1(&"No animation found".into());
            return Ok(());
        }

        if self.frame_index >= self.lipsync.animation_frame_count(self.animation_index) {
            self.playback_state = PlaybackState::Stopped;
            return Ok(());
        };

        self.lipsync.draw_animation_frame(
            &mut self.framebuffer,
            &self.sprite_sheet,
            self.animation_index,
            self.frame_index,
        );

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

        for y in 0..200 {
            for x in 0..320 {
                let c = self.framebuffer.get(x, y);
                let rgb = self.pal.get_rgb888(c as usize);
                let alpha = 255; //self.framebuffer.get_is_set(x, y) as u8 * 255;

                let ofs = 4 * (y * 320 + x) as usize;
                self.image[ofs + 0] = rgb.0;
                self.image[ofs + 1] = rgb.1;
                self.image[ofs + 2] = rgb.2;
                self.image[ofs + 3] = alpha;
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

use cpal::{
    FromSample, SizedSample, Stream,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

#[allow(dead_code)]
#[wasm_bindgen]
pub struct Handle(Stream);

#[wasm_bindgen]
pub fn beep() -> Handle {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();

    Handle(match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
        // not all supported sample formats are included in this example
        _ => panic!("Unsupported sample format!"),
    })
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Stream
where
    T: SizedSample + FromSample<f32>,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| console::error_1(&format!("an error occurred on stream: {err}").into());

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| write_data(data, channels, &mut next_value),
            err_fn,
            None,
        )
        .unwrap();
    stream.play().unwrap();
    stream
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
