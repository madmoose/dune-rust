mod countdown_timer;

use std::process::exit;

use dune::{
    Framebuffer, Palette, Rect, SpriteSheet, draw_sprite, draw_sprite_from_sheet, sprite_blitter,
};

use crate::countdown_timer::CountdownTimer;

const MAX_PARTICLES: usize = 64;
const GLOBAL_Y_OFFSET: i16 = 24;

static ATTACK_HSQ: &[u8] = include_bytes!("../../../assets/ATTACK.HSQ");

struct NightAttackData {
    timer0: CountdownTimer<i8>,  // offset +0
    timer1: CountdownTimer<i8>,  // offset +1
    unk2: u8,                    // offset +2
    unk3: u8,                    // offset +3
    timer4: CountdownTimer<i16>, // offset +4 (word)
    timer6: CountdownTimer<i8>,  // offset +6
    timer7: CountdownTimer<i8>,  // offset +7
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Particle {
    pub rect: Rect,     // offset +0
    pub sprite_id: u16, // offset +8
    pub subtype: u16,   // offset +10
    pub flags: u8,      // offset +12
    pub data0: u8,      // offset +13
    pub data1: u8,      // offset +14
    pub _data2: u8,     // offset +15
    pub _data3: u8,     // offset +16
                        // Total size 17
}

trait Words {
    fn hi(&self) -> u16;
    fn lo(&self) -> u16;
    fn split_words(&self) -> (u16, u16);
}

impl Words for u32 {
    /// Returns the high 16 bits as a u16
    fn hi(&self) -> u16 {
        (*self >> 16) as u16
    }

    /// Returns the low 16 bits as a u16
    fn lo(&self) -> u16 {
        *self as u16
    }

    /// Returns both high and low words as a tuple (high, low)
    fn split_words(&self) -> (u16, u16) {
        (self.hi(), self.lo())
    }
}

trait WordJoin {
    fn join_words(&self) -> u32;
}

impl WordJoin for (u16, u16) {
    /// Combines two u16 values into a u32, with the first element as high word
    /// and the second element as low word
    fn join_words(&self) -> u32 {
        ((self.0 as u32) << 16) | (self.1 as u32)
    }
}

fn main() {
    let sprite_sheet = SpriteSheet::from_possibly_compressed_slice(ATTACK_HSQ).unwrap();

    let mut pal = Palette::new();
    sprite_sheet.apply_palette_update(&mut pal).unwrap();

    let screen_pal = pal.clone();

    let mut framebuffer = Framebuffer::new(320, 200);
    for y in 0..200 {
        for x in 0..320 {
            framebuffer.set(x, y, 1);
        }
    }

    let mut x = 0;
    let y = 0;
    while x < 320 {
        let sprite = sprite_sheet.get_sprite(2).unwrap();
        draw_sprite(sprite, x as i16, y + GLOBAL_Y_OFFSET, &mut framebuffer).unwrap();
        x += sprite.width()
    }

    let mut x = 0;
    let y = 81;
    while x < 320 {
        let sprite = sprite_sheet.get_sprite(3).unwrap();
        draw_sprite(sprite, x as i16, y + GLOBAL_Y_OFFSET, &mut framebuffer).unwrap();
        x += sprite.width();
    }

    let sprite_list = [(49, 0, 76), (1, 0, 134)];
    for (id, x, y) in sprite_list {
        draw_sprite_from_sheet(&sprite_sheet, id, x, y + GLOBAL_Y_OFFSET, &mut framebuffer)
            .unwrap();
    }

    let bg_framebuffer = framebuffer.clone();

    let mut game_state = GameState {
        bg_framebuffer,
        framebuffer: &mut framebuffer,
        pal_2bf: pal,
        pal_5bf: Palette::new(),
        screen_pal,
        sprite_sheet,
        word_2316e_particle_count: 0,
        unk_23170_particles: [Particle::default(); MAX_PARTICLES],
        byte_1f59a: 0,
        word_1f4b0_rand_bits: 0x7302,
        word_20a42: [0x7d, 0x65, 0x64, 0x65, 0xef, 0x7a, 0x10f, 0x7d],
        // from seg001:15a2
        word_20a52: [0xfa04, 0xfc06, 0xfcfa, 0xfafc],
        byte_23b9b: 0,
        byte_23bea: 0,
        word_23c4e: 0,
        word_2ccd4_rand_seed: 0x01d2,
        word_2ccd6_rand_seed: 0x0273,
        night_attack_data: NightAttackData {
            timer0: CountdownTimer::new("timer0", 0, 0),
            timer1: CountdownTimer::new("timer1", 0, 0),
            unk2: 0,
            unk3: 0,
            timer4: CountdownTimer::new("timer4", 0, 0),
            timer6: CountdownTimer::new("timer6", 0, 0),
            timer7: CountdownTimer::new("timer7", 0, 17),
        },
    };

    let mut frame_number = 0;
    loop {
        println!(
            "#####+ frame {} [ds:0000]={:04x} [ds:d824]={:04x} [ds:d826]={:04x}            {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
            frame_number,
            game_state.word_1f4b0_rand_bits,
            game_state.word_2ccd4_rand_seed,
            game_state.word_2ccd6_rand_seed,
            game_state.night_attack_data.timer0.get(),
            game_state.night_attack_data.timer1.get(),
            game_state.night_attack_data.unk2,
            game_state.night_attack_data.unk3,
            game_state.night_attack_data.timer4.get() & 0xff,
            (game_state.night_attack_data.timer4.get() >> 8) as u8,
            game_state.night_attack_data.timer6.get(),
            game_state.night_attack_data.timer7.get(),
        );
        game_state.sub_10b45();

        game_state
            .framebuffer
            .write_ppm_scaled(
                &game_state.screen_pal,
                &format!("ppm/night_attack-{frame_number:05}.ppm"),
                5,
                6,
            )
            .unwrap();
        frame_number += 1;
        if frame_number > 659 {
            exit(0);
        }
    }
}

// Global state structure
pub struct GameState<'a> {
    bg_framebuffer: Framebuffer,
    framebuffer: &'a mut Framebuffer,
    pal_2bf: Palette,
    pal_5bf: Palette,
    screen_pal: Palette,
    sprite_sheet: SpriteSheet,
    word_1f4b0_rand_bits: u16,
    byte_1f59a: i8,
    word_20a42: [i16; 8],
    word_20a52: [u16; 4],
    word_2316e_particle_count: u16,
    unk_23170_particles: [Particle; MAX_PARTICLES],
    byte_23b9b: u8,
    byte_23bea: u8,
    word_23c4e: u16,
    word_2ccd4_rand_seed: u16,
    word_2ccd6_rand_seed: u16,
    night_attack_data: NightAttackData,
}

impl<'a> GameState<'a> {
    fn particle(&mut self, index: u16) -> &mut Particle {
        &mut self.unk_23170_particles[index as usize]
    }

    fn sub_1e3b7_rand_masked(&mut self, mask: u16) -> u16 {
        let (dx, ax) = (self.word_2ccd4_rand_seed as u32 * 0x0e56d).split_words();
        let ax = ax.wrapping_add(1);

        self.word_2ccd4_rand_seed = ax;

        let ax = ((dx, ax).join_words() >> 8) as u16;

        ax & mask
    }

    fn sub_1e3cc_rand(&mut self) -> u16 {
        let (dx, ax) = (self.word_2ccd6_rand_seed as u32 * 0xcbd1).split_words();

        let ax = ax.wrapping_add(1);

        self.word_2ccd6_rand_seed = ax;

        ((dx, ax).join_words() >> 8) as u16
    }

    fn sub_10b45(&mut self) {
        self.night_attack_data.timer7.tick();

        if self.byte_1f59a <= 0 && self.night_attack_data.timer7.triggered() {
            self.sub_10d0d(self.night_attack_data.timer7.get() == 16);
        }

        if self.word_23c4e != 0 || self.byte_23b9b != 0 {
            return;
        }

        if self.night_attack_data.timer4.tick() {
            if self.night_attack_data.timer6.tick() {
                let random_val = self.sub_1e3cc_rand();

                self.night_attack_data.timer6.set((random_val & 0x7F) as i8);
                self.night_attack_data.timer4.set((random_val >> 8) as i16);
            } else {
                let random_val = self.sub_1e3cc_rand();

                let bx = random_val;
                let mut dx = random_val;

                let bx = (bx & 0x7F) as i16;

                if (bx & 0xFF) >= 0x60 {
                    // Skip particle spawn
                } else if (bx & 0xFF) >= 0x30 {
                    dx = ((dx & 0xFF) << 8) | ((dx >> 8) & 0xFF); // swap bytes

                    let mut dh = (dx >> 8) as u8;
                    dh = dh.rotate_left(1);
                    dx = ((dh as u16) << 8) | (dx & 0x00ff);

                    let dx = dx & 0x1FF;

                    let dx = dx as i16;

                    if dx < 320 {
                        let sprite_id = ((bx as u16) & 7) + 28;

                        self.sub_1c60b_particles_spawn_particle(sprite_id, dx, bx, 0);
                    }
                }
            }
        }

        if self.night_attack_data.timer0.tick() {
            self.sub_10c3b();
        }

        let particle_count = self.word_2316e_particle_count;
        if particle_count == 0 {
            return;
        }

        // Loop through particles

        let mut i: u16 = 0;
        while i < particle_count {
            let mut ax = self.particle(i).sprite_id;
            let mut dx = self.particle(i).subtype;

            if (ax & 0xFF) < 0x14 {
                ax >>= 2;

                self.word_1f4b0_rand_bits = self.word_1f4b0_rand_bits.rotate_left(1);
                ax = (ax << 1) | (self.word_1f4b0_rand_bits & 1);

                self.word_1f4b0_rand_bits = self.word_1f4b0_rand_bits.rotate_left(1);
                ax = (ax << 1) | (self.word_1f4b0_rand_bits & 1);
            } else if (ax & 0xFF) < 0x1C {
                let mut bx =
                    ((self.particle(i).data1 as u16) << 8) | (self.particle(i).data0 as u16);

                self.sub_10cea_word(&mut bx, &mut dx);

                self.sub_10cea_word(&mut bx, &mut dx);

                self.particle(i).data0 = (bx & 0xFF) as u8;
                self.particle(i).data1 = ((bx >> 8) & 0xFF) as u8;
                let randv = self.word_1f4b0_rand_bits;

                self.word_1f4b0_rand_bits = randv.rotate_left(3);
                ax = (self.word_1f4b0_rand_bits & 7) + 0x14;
            } else {
                ax = ax.wrapping_add(1);
                if (ax & 0xFF) > 0x2D {
                    self.sub_1c58a_particles_remove_particle(i);

                    i += 1;

                    continue;
                }
            }

            // let particle = &mut self._unk_23170_particles[i as usize];
            self.particle(i).sprite_id = ax;

            let bl = ((dx >> 8) & 0xFF) as i8;
            let bx_extended = bl as i16;

            let dx_extended = (dx & 0xFF) as i8 as i16;

            self.sub_1c661(dx_extended, bx_extended, i);

            let mut should_remove = (self.particle(i).rect.x0 as u16) >= 320;
            if !should_remove {
                should_remove = self.particle(i).rect.x1 < 0;
            }

            if !should_remove {
                should_remove = self.particle(i).rect.y1 < 0;
            }

            if should_remove {
                self.sub_1c58a_particles_remove_particle(i);
            }

            i += 1;
        }
    }

    fn sub_10c3b(&mut self) {
        let mut ax;
        if self.night_attack_data.timer1.tick() {
            if (self.word_1f4b0_rand_bits & 3) == 0 {
                self.night_attack_data.timer7.set(11);

                if (self.word_1f4b0_rand_bits & 0x0C) == 0 {
                    self.night_attack_data.timer7.set(17);
                }
            }

            ax = self.sub_1e3cc_rand();

            if self.byte_23bea != 0 {
                ax &= 0xFFEF; // Clear bit 4 in al
            }

            let mut cx = ax;

            let masked_random = self.sub_1e3b7_rand_masked(7);

            self.night_attack_data
                .timer1
                .set((masked_random & 0xff) as i8);

            if (masked_random & 0xFF) >= 4 {
                cx |= 0x4000; // Set bit 6 in ch
            }

            self.night_attack_data.unk2 = (cx & 0xFF) as u8;
            self.night_attack_data.unk3 = ((cx >> 8) & 0xFF) as u8;
        }

        self.night_attack_data.timer0.set(8);

        let mut ax = self.night_attack_data.unk2 as u16;
        let di = ax;

        let bl = self.night_attack_data.unk3;

        ax &= 0x10;
        if ax == 0 {
            let bx_masked: usize = (bl & 6) as usize;

            let si = self.word_20a52[bx_masked >> 1];

            let bx_doubled = bx_masked * 2;

            ax += bx_doubled as u16;

            ax = (ax & 0xFF00) | (((ax & 0xFF) + 4) & 0xFF);

            self.spawn_particle_final(ax, di, si);
        } else {
            let mut al = bl;
            let mut ah = bl;

            let mut ax = ((ah as u16) << 8) | (al as u16);
            ax &= 0xC03F;
            al = (ax & 0xFF) as u8;
            ah = ((ax >> 8) & 0xFF) as u8;

            if (ah & 0x40) != 0 {
                self.word_1f4b0_rand_bits = self.word_1f4b0_rand_bits.rotate_left(1);
                if (self.word_1f4b0_rand_bits & 1) != 0 {
                    let mut cl: u8 = 0x0A;

                    if (ah & 0x80) != 0 {
                        cl = cl.wrapping_neg();
                    }

                    al = al.wrapping_add(cl);

                    if (al & 0x80) != 0 {
                        ah ^= 0x80;

                        al = 0;
                    }

                    if al >= 0x40 {
                        al = 0x3F;

                        ah ^= 0x80;
                    }

                    ah |= al;

                    self.night_attack_data.unk3 = ah;
                }
            }

            al = al.wrapping_add(0xE0);

            let (bx, dx) = self.sub_15198(al);

            let dl = (dx as u16) & 0x00ff;
            let bl = bx & 0x00ff;
            let si = (bl << 8) | dl;

            let ax = 0x14;

            self.spawn_particle_final(ax, di, si);
        }
    }

    fn sub_15198(&mut self, value: u8) -> (u16, i16) {
        let mut bx = value as u16;

        let bl = (bx as u8).wrapping_add(0x20);

        let mut bh = bl;

        bh &= 0x7F;

        let (bx, dx) = if bh < 0x40 {
            bx = 0xFFE0;
            if (bl as i8) >= 0 {
                let dx = (value as i8) as i16;

                (bx, dx)
            } else {
                let mut al = value.wrapping_sub(0x80);

                al = al.wrapping_neg();

                bx = (-(bx as i16)) as u16;

                let dx = (al as i8) as i16;

                (bx, dx)
            }
        } else {
            let mut dx = 0x20i16;

            let mut al = value.wrapping_sub(0x40);

            if (bl as i8) < 0 {
                dx = -dx;

                al = al.wrapping_sub(0x80);

                al = al.wrapping_neg();
            }

            bx = (al as i8) as i16 as u16;

            (bx, dx)
        };

        (bx, dx)
    }

    fn spawn_particle_final(&mut self, ax: u16, di: u16, si: u16) {
        let di_masked = di & 0x0C;
        let bx = self.word_20a42[((di_masked >> 1) + 1) as usize]; // word array
        let dx = self.word_20a42[(di_masked >> 1) as usize]; // word array

        if self
            .sub_1c60b_particles_spawn_particle(ax, dx, bx, si)
            .is_some()
        {
            let last_particle_idx = (self.word_2316e_particle_count - 1) as usize;
            self.unk_23170_particles[last_particle_idx].data0 = 0;
            self.unk_23170_particles[last_particle_idx].data1 = 0;
        }
    }

    fn sub_10cea_word(&self, bx: &mut u16, dx: &mut u16) {
        let mut bl = (*bx & 0xFF) as i8;
        let mut bh = ((*bx >> 8) & 0xFF) as i8;
        let mut dl = (*dx & 0xFF) as i8;
        let mut dh = ((*dx >> 8) & 0xFF) as i8;

        self.sub_10cea(&mut bl, &mut bh, &mut dl, &mut dh);

        *bx = ((bh as u8 as u16) << 8) | (bl as u8 as u16);
        *dx = ((dh as u8 as u16) << 8) | (dl as u8 as u16);
    }

    fn sub_10cea(&self, bl: &mut i8, bh: &mut i8, dl: &mut i8, dh: &mut i8) {
        let mut al = *dl;

        if al < 0 {
            al = al.wrapping_neg();

            self.sub_10cf2(bl, bh, dl, dh, al);

            *dh = dh.wrapping_neg();
        } else {
            self.sub_10cf2(bl, bh, dl, dh, al);
        }
    }

    fn sub_10cf2(&self, bl: &mut i8, bh: &mut i8, dl: &mut i8, dh: &mut i8, mut al: i8) {
        al = al.wrapping_add(*bl);

        let mut ax = (al as u8) as u16;
        ax = ax.rotate_right(5);

        let al2 = ax as u8;
        let ah2 = (ax >> 8) as u8;

        *bl = (ah2 >> 3) as i8;
        *dl = al2 as i8;

        core::mem::swap(bl, bh);
        core::mem::swap(dl, dh);
    }

    fn sub_10d0d(&mut self, flag: bool) {
        // let al = self.night_attack_data.timer7;
        // let bx = 384u16;
        // let cx = 84u16;
        let mut dl = 55u8;

        if !flag {
            dl -= 1;

            if self.night_attack_data.timer7.get() != 10 {
                // self.gfx_vtable_func_39_transition_palette(al, bx, cx, dl);

                for i in 128..128 + 0x1c {
                    let divisor = self.night_attack_data.timer7.get().max(1) as i16;

                    let r = (self.pal_2bf.get(i).0 as i16 - self.pal_5bf.get(i).0 as i16) / divisor
                        + self.pal_5bf.get(i).0 as i16;
                    let g = (self.pal_2bf.get(i).1 as i16 - self.pal_5bf.get(i).1 as i16) / divisor
                        + self.pal_5bf.get(i).1 as i16;
                    let b = (self.pal_2bf.get(i).2 as i16 - self.pal_5bf.get(i).2 as i16) / divisor
                        + self.pal_5bf.get(i).2 as i16;

                    let rgb = (r as u8, g as u8, b as u8);

                    self.pal_5bf.set(i, rgb);
                    self.screen_pal.set(i, rgb);
                }

                return;
            }
        }

        self.sub_1c13b_open_onmap_resource();
        let ax = dl as u16;

        let si = self.sprite_sheet.get_resource(ax).unwrap();
        let dsdx = &si[6..];

        for i in 0..28 {
            let rgb = (dsdx[3 * i], dsdx[3 * i + 1], dsdx[3 * i + 2]);
            self.pal_5bf.set(128 + i, rgb);
            self.screen_pal.set(128 + i, rgb);
        }

        let si = self.sprite_sheet.get_resource(53).unwrap();

        let dsdx = &si[6..];

        for i in 0..28 {
            let rgb = (dsdx[3 * i], dsdx[3 * i + 1], dsdx[3 * i + 2]);
            self.pal_2bf.set(128 + i, rgb);
        }
    }

    fn sub_1c202(&self, sprite_id: u16, center_x: &mut i16, center_y: &mut i16) {
        let Some(sprite) = self.sprite_sheet.get_sprite(sprite_id) else {
            return;
        };

        let width = sprite.width();
        let height = sprite.height();

        *center_x = center_x.saturating_sub_unsigned(width / 2);
        *center_y = center_y.saturating_sub_unsigned(height / 2);
    }

    fn sub_1c60b_particles_spawn_particle(
        &mut self,
        sprite_id: u16,
        center_x: i16,
        center_y: i16,
        subtype: u16,
    ) -> Option<&mut Particle> {
        self.sub_1c13b_open_onmap_resource();

        let mut x0 = center_x;
        let mut y0 = center_y;

        self.sub_1c202(sprite_id, &mut x0, &mut y0);

        self.unk_23170_particles[self.word_2316e_particle_count as usize]
            .rect
            .x0 = x0;

        self.unk_23170_particles[self.word_2316e_particle_count as usize]
            .rect
            .y0 = y0;

        self.unk_23170_particles[self.word_2316e_particle_count as usize].sprite_id = sprite_id;

        self.unk_23170_particles[self.word_2316e_particle_count as usize].subtype = subtype;

        self.unk_23170_particles[self.word_2316e_particle_count as usize].flags = 0;

        let sprite = self.sprite_sheet.get_sprite(sprite_id).unwrap();

        let width = sprite.width();
        let height = sprite.height();

        self.unk_23170_particles[self.word_2316e_particle_count as usize]
            .rect
            .x1 = x0.saturating_add_unsigned(width);
        self.unk_23170_particles[self.word_2316e_particle_count as usize]
            .rect
            .y1 = y0.saturating_add_unsigned(height);

        self.word_2316e_particle_count += 1;

        Some(&mut self.unk_23170_particles[self.word_2316e_particle_count as usize])
    }

    fn sub_1c661(&mut self, dx: i16, bx: i16, particle_index: u16) {
        let particle_index = particle_index as usize;

        self.sub_1c13b_open_onmap_resource();

        let mut temp_rect = self.unk_23170_particles[particle_index].rect;

        self.unk_23170_particles[particle_index].rect.x0 = self.unk_23170_particles[particle_index]
            .rect
            .x0
            .wrapping_add(dx);

        self.unk_23170_particles[particle_index].rect.y0 = self.unk_23170_particles[particle_index]
            .rect
            .y0
            .wrapping_add(bx);

        self.unk_23170_particles[particle_index].rect.x1 = self.unk_23170_particles[particle_index]
            .rect
            .x1
            .wrapping_add(dx);

        self.unk_23170_particles[particle_index].rect.y1 = self.unk_23170_particles[particle_index]
            .rect
            .y1
            .wrapping_add(bx);

        if dx >= 0 {
            temp_rect.x1 = self.unk_23170_particles[particle_index].rect.x1;
        } else {
            temp_rect.x0 = self.unk_23170_particles[particle_index].rect.x0;
        }

        if bx >= 0 {
            temp_rect.y1 = self.unk_23170_particles[particle_index].rect.y1;
        } else {
            temp_rect.y0 = self.unk_23170_particles[particle_index].rect.y0;
        }

        self.sub_1c6ad_particles_update_dirty_rect(&temp_rect);
    }

    fn sub_1c13b_open_onmap_resource(&self) {}

    fn sub_1c58a_particles_remove_particle(&mut self, index: u16) {
        self.sub_1c13b_open_onmap_resource();

        if self.word_2316e_particle_count != 0 && index < self.word_2316e_particle_count {
            self.unk_23170_particles[index as usize].flags |= 0x80;
            let rect = self.unk_23170_particles[index as usize].rect;
            self.sub_1c6ad_particles_update_dirty_rect(&rect);

            if index < self.word_2316e_particle_count - 1 {
                for i in index..self.word_2316e_particle_count - 1 {
                    self.unk_23170_particles[i as usize] = self.unk_23170_particles[i as usize + 1];
                }
            }

            self.word_2316e_particle_count -= 1;

            // for i in 0..2 {
            //     if false {
            //         // TODO
            //     }
            // }
        }
    }

    // fn gfx_vtable_func_39_transition_palette(&mut self, al: u8, bx: u16, cx: u16, dl: u8) {
    //     let speed = u16::max(al as u16, 1);
    //     let offset = bx;
    //     let count = cx;

    //     for i in 0..count {
    //         // let d = src[i] - dst[i];
    //         // dst[i] += d / speed;
    //     }
    // }

    fn sub_1c6ad_particles_update_dirty_rect(&mut self, dirty_rect: &Rect) {
        self.sub_1c13b_open_onmap_resource();

        let screen_bounds = Rect {
            x0: 0,
            y0: 0,
            x1: 320,
            y1: 200,
        };

        let rect = dirty_rect.clip(&screen_bounds);

        for y in rect.y0..rect.y1 {
            for x in rect.x0..rect.x1 {
                let x = x as u16;
                let y = (y + GLOBAL_Y_OFFSET) as u16;
                let c = self.bg_framebuffer.get(x, y);
                self.framebuffer.set(x, y, c);
            }
        }

        for i in 0..self.word_2316e_particle_count {
            let particle = &self.unk_23170_particles[i as usize];
            if particle.flags & 0x80 != 0 {
                continue;
            }

            if particle.rect.clip(dirty_rect).is_empty() {
                continue;
            }

            let clip_rect = Rect {
                x0: 0,
                y0: GLOBAL_Y_OFFSET,
                x1: 320,
                y1: 200,
            };

            if let Some(sprite) = self.sprite_sheet.get_sprite(particle.sprite_id) {
                sprite_blitter(sprite, self.framebuffer)
                    .at(particle.rect.x0, particle.rect.y0 + GLOBAL_Y_OFFSET)
                    .clip_rect(clip_rect)
                    .draw()
                    .unwrap();
            };
        }
    }
}
