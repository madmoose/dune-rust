#![allow(unused)]

use std::{io::Cursor, process::exit};

mod countdown_timer;

use dune::{Rect, hsq};
use framebuffer::{Framebuffer, Palette};
use sprite::SpriteSheet;

use crate::countdown_timer::CountdownTimer;

macro_rules! ln {
    ($self:expr, $ip:expr, $inst:expr) => {
        $self.ln($ip, $inst, None)
    };
    ($self:expr, $ip:expr, $inst:expr, $mem_ref:expr) => {
        $self.ln($ip, $inst, Some($mem_ref))
    };
}

static ATTACK_HSQ: &[u8] = include_bytes!("../../assets/ATTACK.HSQ");
const MAX_PARTICLES: usize = 64;

fn read_sprite_sheet(data: &[u8]) -> std::io::Result<SpriteSheet> {
    let mut reader = Cursor::new(data);
    let header = hsq::Header::from_reader(&mut reader)?;

    if !header.is_compressed() {
        return SpriteSheet::from_slice(data);
    }

    if header.compressed_size() as usize != data.len() {
        println!("Packed length does not match resource size");
        return SpriteSheet::from_slice(data);
    }

    let mut unpacked_data = vec![0; header.uncompressed_size() as usize];
    let mut writer = Cursor::new(&mut unpacked_data);

    hsq::unhsq(reader, &mut writer)?;

    SpriteSheet::from_slice(&unpacked_data)
}

struct NightAttackData {
    timer0: i8,  // offset +0
    timer1: i8,  // offset +1
    unk2: u8,    // offset +2
    unk3: u8,    // offset +3
    timer4: i16, // offset +4 (word)
    timer6: i8,  // offset +6
    timer7: u8,  // offset +7
}

#[derive(Copy, Clone, Debug, Default)]
struct Particle {
    rect: Rect,     // offset +0
    sprite_id: u16, // offset +8
    subtype: u16,   // offset +10
    flags: u8,      // offset +12
    data0: u8,      // offset +13
    data1: u8,      // offset +14
    data2: u8,      // offset +15
    data3: u8,      // offset +16
                    // Total size 17
}

const GLOBAL_Y_OFFSET: u16 = 24;

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
    let sprite_sheet = read_sprite_sheet(ATTACK_HSQ).unwrap();

    let mut pal = Palette::new();
    sprite_sheet.apply_palette_update(&mut pal).unwrap();

    let screen_pal = pal.clone();

    let mut framebuffer = framebuffer::Framebuffer::new(320, 200);
    for y in 0..200 {
        for x in 0..320 {
            framebuffer.put_pixel(x, y, 1);
        }
    }

    let mut x = 0;
    let y = 0;
    while x < 320 {
        let sprite = sprite_sheet.get_sprite(2).unwrap();
        sprite
            .draw(x as i16, (GLOBAL_Y_OFFSET + y) as i16, &mut framebuffer)
            .unwrap();
        x += sprite.width()
    }

    x = 0;
    let y = 81;
    while x < 320 {
        let sprite = sprite_sheet.get_sprite(3).unwrap();
        sprite
            .draw(x as i16, (GLOBAL_Y_OFFSET + y) as i16, &mut framebuffer)
            .unwrap();
        x += sprite.width()
    }

    let sprite_list = [(49, 0, 76), (1, 0, 134)];
    for (id, x, y) in sprite_list {
        sprite_sheet.draw_sprite(id, x, (GLOBAL_Y_OFFSET + y) as i16, &mut framebuffer);
    }

    let bg_framebuffer = framebuffer.clone();

    // seg001:1592 word_20A42      dw  7Dh                  ; DATA XREF: sub_10C3B+A2r
    // seg001:1594 word_20A44      dw  65h                  ; DATA XREF: sub_10C3B+9Er
    // seg001:1596                 dw  64h
    // seg001:1598                 dw  65h
    // seg001:159A                 dw  EFh
    // seg001:159C                 dw  7Ah
    // seg001:159E                 dw 10Fh
    // seg001:15A0                 dw  7Dh

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
            timer0: 0,
            timer1: 0,
            unk2: 0,
            unk3: 0,
            timer4: 0,
            timer6: 0,
            timer7: 0,
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
            game_state.night_attack_data.timer0,
            game_state.night_attack_data.timer1,
            game_state.night_attack_data.unk2,
            game_state.night_attack_data.unk3,
            game_state.night_attack_data.timer4 & 0xff,
            (game_state.night_attack_data.timer4 >> 8) as u8,
            game_state.night_attack_data.timer6,
            game_state.night_attack_data.timer7,
        );
        game_state.sub_10b45();

        println!(
            "#####- frame {} [ds:0000]={:04x} [ds:d824]={:04x} [ds:d826]={:04x}",
            frame_number,
            game_state.word_1f4b0_rand_bits,
            game_state.word_2ccd4_rand_seed,
            game_state.word_2ccd6_rand_seed,
        );

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

        game_state.dump_particles(frame_number);

        for i in 0..game_state.word_2316e_particle_count {
            println!("\t{i:3} {:?}", game_state.particle(i));
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

enum MemRef {
    Db(u16),
    Dw(u16),
}

enum Val {
    Db(u8),
    Dw(u16),
}

impl<'a> GameState<'a> {
    fn particle(&mut self, index: u16) -> &mut Particle {
        &mut self.unk_23170_particles[index as usize]
    }

    fn get_byte_val(&self, ofs: u16) -> Option<u8> {
        let b = match ofs {
            0x3cc0..0x3dc0 => {
                /*
                    rect: Rect,     // offset +0
                    sprite_id: u16, // offset +8
                    subtype: u16,   // offset +10
                    flags: u8,      // offset +12
                    data0: u8,      // offset +13
                    data1: u8,      // offset +14
                    data2: u8,      // offset +15
                    data3: u8,      // offset +16
                                    // Total size 17
                */
                let idx = ((ofs - 0x3cc0) / 0x11) as usize;
                match ((ofs - 0x3cc0) % 0x11) {
                    12 => self.unk_23170_particles[idx].flags,
                    13 => self.unk_23170_particles[idx].data0,
                    14 => self.unk_23170_particles[idx].data1,
                    15 => self.unk_23170_particles[idx].data2,
                    16 => self.unk_23170_particles[idx].data3,
                    _ => return None,
                }
            }
            /*
               timer0: i8,  // offset +0
               timer1: i8,  // offset +1
               unk2: u8,    // offset +2
               unk3: u8,    // offset +3
               timer4: i16, // offset +4 (word)
               timer6: i8,  // offset +6
               timer7: u8,  // offset +7
            */
            0x4856 => self.night_attack_data.timer0 as u8,
            0x4857 => self.night_attack_data.timer1 as u8,
            0x4858 => self.night_attack_data.unk2,
            0x4859 => self.night_attack_data.unk3,
            0x485a => self.night_attack_data.timer4 as u16 as u8,
            0x485b => ((self.night_attack_data.timer4 as u16) >> 8) as u8,
            0x485c => self.night_attack_data.timer6 as u8,
            0x485d => self.night_attack_data.timer7,
            _ => return None,
        };
        Some(b)
    }

    fn get_word_val(&self, ofs: u16) -> Option<u16> {
        // println!("get_word_val({ofs:#04x})");
        let w = match ofs {
            0x0000 => self.word_1f4b0_rand_bits,
            0x1592 => self.word_20a42[0] as u16,
            0x1594 => self.word_20a42[1] as u16,
            0x1596 => self.word_20a42[2] as u16,
            0x1598 => self.word_20a42[3] as u16,
            0x159a => self.word_20a42[4] as u16,
            0x159c => self.word_20a42[5] as u16,
            0x159e => self.word_20a42[6] as u16,
            0x15a0 => self.word_20a42[7] as u16,
            0x3cbe => self.word_2316e_particle_count,
            0x3cc0..0x3dc0 => {
                /*
                    rect: Rect,     // offset +0
                    sprite_id: u16, // offset +8
                    subtype: u16,   // offset +10
                    flags: u8,      // offset +12
                    data0: u8,      // offset +13
                    data1: u8,      // offset +14
                    data2: u8,      // offset +15
                    data3: u8,      // offset +16
                                    // Total size 17
                */
                let idx = ((ofs - 0x3cc0) / 0x11) as usize;
                match ((ofs - 0x3cc0) % 0x11) {
                    0 => self.unk_23170_particles[idx].rect.x0 as u16,
                    2 => self.unk_23170_particles[idx].rect.y0 as u16,
                    4 => self.unk_23170_particles[idx].rect.x1 as u16,
                    6 => self.unk_23170_particles[idx].rect.y1 as u16,
                    8 => self.unk_23170_particles[idx].sprite_id,
                    10 => self.unk_23170_particles[idx].subtype,
                    13 => {
                        (self.unk_23170_particles[idx].data0 as u16)
                            | ((self.unk_23170_particles[idx].data1 as u16) << 8)
                    }
                    _ => return None,
                }
            }
            0xd824 => self.word_2ccd4_rand_seed,
            0xd826 => self.word_2ccd6_rand_seed,
            0x485a => self.night_attack_data.timer4 as u16,
            _ => return None,
        };
        Some(w)
    }

    fn ln(&self, ip: u16, inst: &str, mem_ref: Option<MemRef>) {
        let s_addr = format!("017e:{ip:04x}");
        let s_inst = inst;

        let (mem_val, mem_ofs) = match mem_ref {
            Some(MemRef::Db(ofs)) => (self.get_byte_val(ofs).map(Val::Db), ofs),
            Some(MemRef::Dw(ofs)) => (self.get_word_val(ofs).map(Val::Dw), ofs),
            None => (None, 0),
        };

        let s_mem = if mem_ref.is_some() {
            match mem_val {
                Some(Val::Db(val)) => format!("[10c9:{mem_ofs:04x}] = {val:02x}"),
                Some(Val::Dw(val)) => format!("[10c9:{mem_ofs:04x}] = {val:04x}"),
                None => format!("[10c9:{mem_ofs:04x}] = ?"),
            }
        } else {
            String::new()
        };

        let it = 0x4856..(0x4856 + 8);
        let s_timers = it
            .map(|i| format!("{:02x}", self.get_byte_val(i).unwrap()))
            .collect::<Vec<_>>()
            .join(" ");

        let s_rands = [0x000, 0xd824, 0xd826]
            .map(|i| format!("{:04x}", self.get_word_val(i).unwrap()))
            .into_iter()
            .collect::<Vec<_>>()
            .join(" ");

        let s = format!("{s_addr} {s_inst:-50} {s_mem:-24} [{s_timers} | {s_rands}]");
        println!("{}", s.trim());
    }

    fn sub_1e3b7_rand_masked(&mut self, mask: u16) -> u16 {
        ln!(self, 0xe3b7, "push    dx");
        ln!(self, 0xe3b8, "mov     ax, [0d824h]", MemRef::Dw(0xd824));
        ln!(self, 0xe3bb, "mov     dx, 0e56dh");
        ln!(self, 0xe3be, "mul     dx");
        let (dx, ax) = (self.word_2ccd4_rand_seed as u32 * 0x0e56d).split_words();

        ln!(self, 0xe3c0, "inc     ax");
        let ax = ax.wrapping_add(1);

        self.word_2ccd4_rand_seed = ax;
        ln!(self, 0xe3c1, "mov     [0d824h], ax", MemRef::Dw(0xd824));

        ln!(self, 0xe3c4, "mov     al, ah");
        ln!(self, 0xe3c6, "mov     ah, dl");
        let ax = ((dx, ax).join_words() >> 8) as u16;

        ln!(self, 0xe3c8, "and     ax, bx");
        ln!(self, 0xe3ca, "pop     dx");

        let ax = ax & mask;
        ln!(self, 0xe3cb, "ret");
        // println!("=====");

        ax
    }

    fn sub_1e3cc_rand(&mut self) -> u16 {
        ln!(self, 0xe3cc, "push    dx");
        ln!(self, 0xe3cd, "mov     ax, [0d826h]", MemRef::Dw(0xd826));
        ln!(self, 0xe3d0, "mov     dx, 0cbd1h");
        ln!(self, 0xe3d3, "mul     dx");
        let (dx, ax) = (self.word_2ccd6_rand_seed as u32 * 0xcbd1).split_words();

        ln!(self, 0xe3d5, "inc     ax");
        let ax = ax.wrapping_add(1);

        ln!(self, 0xe3d6, "mov     [0d826h], ax", MemRef::Dw(0xd826));
        self.word_2ccd6_rand_seed = ax;

        ln!(self, 0xe3d9, "mov     al, ah");
        ln!(self, 0xe3db, "mov     ah, dl");

        let ax = ((dx, ax).join_words() >> 8) as u16;

        ln!(self, 0xe3dd, "pop     dx");
        ln!(self, 0xe3de, "ret");
        // println!("=====");

        ax
    }

    fn sub_10b45(&mut self) {
        ln!(
            self,
            0x0b45,
            "mov     si, offset _unk_23D06_night_attack_data"
        );
        ln!(
            self,
            0x0b48,
            "dec     byte ptr [si+7h]",
            MemRef::Db(0x4856 + 7)
        );

        self.night_attack_data.timer7 = self.night_attack_data.timer7.wrapping_sub(1);

        ln!(self, 0x0b4b, "cmp     byte_1F59A, 0");
        ln!(self, 0x0b50, "jg      short loc_10B5F");
        if self.byte_1f59a <= 0 {
            ln!(self, 0x0b52, "mov     bl, [si+7h]");
            ln!(self, 0x0b55, "cmp     bl, 10h");
            ln!(self, 0x0b58, "ja      short loc_10B5F");
            if self.night_attack_data.timer7 <= 16 {
                // unsigned comparison
                ln!(self, 0x0b5a, "push    si");
                ln!(self, 0x0b5b, "call    sub_10D0D");
                ln!(self, 0x0b5e, "pop     si");
                self.sub_10d0d(self.night_attack_data.timer7 == 16);
            }
        }

        ln!(self, 0x0b5f, "mov     ax, word_23C4E");
        ln!(self, 0x0b62, "or      al, byte_23B9B");
        ln!(self, 0x0b66, "or      ax, ax");
        ln!(self, 0x0b68, "jz      short loc_10B6B");
        if self.word_23c4e != 0 || self.byte_23b9b != 0 {
            ln!(self, 0x0b6a, "ret");
            return;
        }

        self.night_attack_data.timer4 = self.night_attack_data.timer4.wrapping_sub(1);
        ln!(self, 0x0b6b, "dec     word ptr [si+4h]", MemRef::Dw(0x485a));
        ln!(self, 0x0b6e, "jns     short loc_10BB7");

        if self.night_attack_data.timer4 < 0 {
            self.night_attack_data.timer6 = self.night_attack_data.timer6.wrapping_sub(1);
            ln!(self, 0x0b70, "dec     byte ptr [si+6h]", MemRef::Db(0x485c));
            ln!(self, 0x0b73, "jns     short loc_10B86");
            if self.night_attack_data.timer6 < 0 {
                ln!(self, 0x0b75, "call    _sub_1E3CC_rand");
                let random_val = self.sub_1e3cc_rand();
                ln!(self, 0x0b78, "and     al, 7fh");
                ln!(self, 0x0b7a, "mov     [si+6h], al");
                self.night_attack_data.timer6 = (random_val & 0x7F) as i8;
                ln!(self, 0x0b7d, "mov     al, ah");
                ln!(self, 0x0b7f, "xor     ah, ah");
                ln!(self, 0x0b81, "mov     [si+4h], ax");
                self.night_attack_data.timer4 = (random_val >> 8) as i16;
                ln!(self, 0x0b84, "jmp     short loc_10BB7");
            } else {
                println!("===== 0x0b86_random_attack_check");
                ln!(self, 0x0b86, "call    _sub_1E3CC_rand");
                let random_val = self.sub_1e3cc_rand();
                ln!(self, 0x0b89, "mov     bx, ax");
                ln!(self, 0x0b8b, "mov     dx, ax");
                let mut bx = random_val;
                let mut dx = random_val;

                ln!(self, 0x0b8d, "and     bx, 7fh");
                let bx = (bx & 0x7F) as i16;
                ln!(self, 0x0b90, "cmp     bl, 60h");
                ln!(self, 0x0b93, "jnb     short loc_10BB7");
                if (bx & 0xFF) >= 0x60 {
                    // Skip particle spawn
                } else {
                    ln!(self, 0x0b95, "cmp     bl, 30h");
                    ln!(self, 0x0b98, "jb      short loc_10BB7");
                    if (bx & 0xFF) >= 0x30 {
                        let old_dx = dx;
                        dx = ((dx & 0xFF) << 8) | ((dx >> 8) & 0xFF); // swap bytes
                        ln!(
                            self,
                            0x0b9a,
                            &format!("xchg    dl, dh   ; dx = {old_dx:04x}->{dx:04x}")
                        );
                        let old_dx = dx;
                        let mut dh = (dx >> 8) as u8;
                        // println!("dh = {dh:02h}");
                        dh = dh.rotate_left(1);
                        dx = ((dh as u16) << 8) | (dx & 0x00ff);
                        // let dh = ((dx >> 8) & 0xFF) << 1; // rol dh, 1
                        // dx = (dx & 0xFF) | ((dh & 0xFF) << 8);
                        ln!(
                            self,
                            0x0b9c,
                            &format!("rol     dh, 1    ; dx = {old_dx:04x}->{dx:04x}")
                        );

                        let old_dx = dx;
                        let dx = (dx & 0x1FF);
                        ln!(
                            self,
                            0x0b9e,
                            &format!("and     dx, 1FFh ; dx = {old_dx:04x}->{dx:04x}")
                        );

                        let dx = dx as i16;
                        ln!(self, 0x0ba2, "cmp     dx, 320");
                        ln!(self, 0x0ba6, "jnb     short loc_10BB7");
                        if dx < 320 {
                            ln!(self, 0x0ba8, "mov     ax, bx");
                            ln!(self, 0x0baa, "and     ax, 7");
                            ln!(self, 0x0bad, "add     ax, 1ch");
                            let sprite_id = ((bx as u16) & 7) + 28;
                            ln!(self, 0x0bb0, "push    si");
                            ln!(self, 0x0bb1, "xor     si, si");
                            ln!(self, 0x0bb3, "call    _sub_1C60B_particles_spawn_particle");
                            self.sub_1c60b_particles_spawn_particle(sprite_id, dx, bx, 0);
                            ln!(self, 0x0bb6, "pop     si");
                        }
                    }
                }
            }
        }

        println!("===== 0x0bb7_handle_particles");
        ln!(self, 0x0bb7, "dec     byte ptr [si]");
        self.night_attack_data.timer0 = self.night_attack_data.timer0.wrapping_sub(1);
        ln!(self, 0x0bb9, "jns     short loc_10BBE");
        if self.night_attack_data.timer0 < 0 {
            ln!(self, 0x0bbb, "call    sub_10C3B");
            self.sub_10c3b();
        }

        println!("===== 0x0bbe");
        ln!(self, 0x0bbe, "mov     di, offset _unk_23170_particles");
        ln!(
            self,
            0x0bc1,
            "mov     cx, [di-2] ; _word_2316E_particle_count"
        );
        ln!(self, 0x0bc4, "jcxz    short locret_10C3A");
        let particle_count = self.word_2316e_particle_count;
        if particle_count == 0 {
            return;
        }

        // Loop through particles

        let mut i: u16 = 0;
        while i < particle_count {
            let mut ax = self.particle(i).sprite_id;
            let mut dx = self.particle(i).subtype;
            ln!(
                self,
                0x0bc6,
                "mov     ax, [di+8]",
                MemRef::Dw(0x03cc0 + i * 0x11 + 8)
            );
            ln!(
                self,
                0x0bc9,
                "mov     dx, [di+0Ah]",
                MemRef::Dw(0x03cc0 + i * 0x11 + 10)
            );

            ln!(self, 0x0bcc, "push    cx");
            ln!(self, 0x0bcd, "cmp     al, 14h");
            ln!(self, 0x0bcf, "jb      short loc_10BF9");
            if (ax & 0xFF) < 0x14 {
                ln!(self, 0x0bf9, "shr     ax, 1");
                ln!(self, 0x0bfb, "shr     ax, 1");
                ax >>= 2;
                // println!("          - ax = {:04x}", ax);

                // println!("          - self.word_1f4b0_rand_bits = {:04x}", self.word_1f4b0_rand_bits);
                ln!(self, 0x0bfd, "rol     word ptr [0h], 1", MemRef::Dw(0x0000));
                // println!("          - self.word_1f4b0_rand_bits = {:04x}", self.word_1f4b0_rand_bits);
                ln!(self, 0x0c01, "rcl     ax, 1");
                self.word_1f4b0_rand_bits = self.word_1f4b0_rand_bits.rotate_left(1);
                ax = (ax << 1) | (self.word_1f4b0_rand_bits & 1);
                // println!("          - ax = {:04x}", ax);
                ln!(self, 0x0c03, "rol     word ptr [0h], 1", MemRef::Dw(0x0000));
                ln!(self, 0x0c07, "rcl     ax, 1");
                self.word_1f4b0_rand_bits = self.word_1f4b0_rand_bits.rotate_left(1);
                ax = (ax << 1) | (self.word_1f4b0_rand_bits & 1);
                // println!("          - ax = {:04x}", ax);
                // println!("          - self.word_1f4b0_rand_bits = {:04x}", self.word_1f4b0_rand_bits);
            } else {
                ln!(self, 0x0bd1, "cmp     al, 1Ch");
                ln!(self, 0x0bd3, "jb      short loc_10BDC");
                if (ax & 0xFF) < 0x1C {
                    ln!(
                        self,
                        0x0bdc,
                        "mov     bx, [di+0Dh] ; data0-data1 as 16-bit value"
                    );
                    let mut bx =
                        ((self.particle(i).data1 as u16) << 8) | (self.particle(i).data0 as u16);

                    // println!("          - bx = {:04x}", bx);
                    // println!("          - dx = {:04x}", dx);

                    ln!(self, 0x0bdf, "call    sub_10CEA");
                    self.sub_10cea_word(&mut bx, &mut dx);

                    // println!("          - bx = {:04x}", bx);
                    // println!("          - dx = {:04x}", dx);

                    ln!(self, 0x0be2, "call    sub_10CEA");
                    self.sub_10cea_word(&mut bx, &mut dx);

                    // println!("          - bx = {:04x}", bx);
                    // println!("          - dx = {:04x}", dx);

                    self.particle(i).data0 = (bx & 0xFF) as u8;
                    self.particle(i).data1 = ((bx >> 8) & 0xFF) as u8;
                    ln!(
                        self,
                        0x0be5,
                        "mov     [di+0Dh], bx",
                        MemRef::Dw(0x3cc0 + i * 0x11 + 0x0d)
                    );

                    ln!(
                        self,
                        0x0be8,
                        "mov     ax, word ptr [0h]",
                        MemRef::Dw(0x0000)
                    );
                    let randv = self.word_1f4b0_rand_bits;
                    // println!("          word_1f4b0_rand_bits = {:04x}", self.word_1f4b0_rand_bits);
                    ln!(self, 0x0beb, "mov     cl, 3");
                    ln!(self, 0x0bed, "rol     ax, cl");
                    self.word_1f4b0_rand_bits = randv.rotate_left(3);
                    // println!("          word_1f4b0_rand_bits = {:04x}", self.word_1f4b0_rand_bits);
                    ln!(
                        self,
                        0x0bef,
                        "mov     word ptr [0h], ax",
                        MemRef::Dw(0x0000)
                    );
                    ln!(self, 0x0bf2, "and     ax, 7");
                    ln!(self, 0x0bf5, "add     al, 14h");
                    ax = (self.word_1f4b0_rand_bits & 7) + 0x14;
                    ln!(self, 0x0bf7, "jmp     0c09h");
                } else {
                    ln!(self, 0x0bd5, "inc     ax");
                    ln!(self, 0x0bd6, "cmp     al, 2Dh");
                    ln!(self, 0x0bd8, "jbe     short loc_10C09");
                    ax = ax.wrapping_add(1);
                    if (ax & 0xFF) > 0x2D {
                        ln!(self, 0x0bda, "jmp     short loc_10C2F (remove particle)");

                        ln!(self, 0x0c2f, "push    di");
                        ln!(self, 0x0c30, "call    _sub_1C58A_particles_remove_particle");
                        self.sub_1c58a_particles_remove_particle(i);
                        ln!(self, 0x0c33, "pop     di");

                        ln!(self, 0x0c34, "pop     cx");
                        ln!(self, 0x0c35, "add     di, 11h ; sizeof(Particle) = 17");
                        i += 1;

                        ln!(self, 0x0c38, "loop    loc_10BC6");

                        continue;
                    }
                }
            }

            // let particle = &mut self._unk_23170_particles[i as usize];
            self.particle(i).sprite_id = ax;
            ln!(
                self,
                0x0c09,
                "mov     [di+8]",
                MemRef::Dw(0x3cc0 + i * 0x11 + 8)
            );

            ln!(self, 0x0c0c, "mov     bl, dh ; subtype high byte");
            ln!(self, 0x0c0e, "mov     ax, bx");
            ln!(self, 0x0c10, "cbw     ; sign extend bl to bx");
            let bl = ((dx >> 8) & 0xFF) as i8;
            let bx_extended = bl as i16;

            ln!(self, 0x0c11, "mov     bx, ax");
            ln!(self, 0x0c13, "mov     ax, dx ; subtype");
            ln!(self, 0x0c15, "cbw     ; sign extend al to ax");
            let dx_extended = (dx & 0xFF) as i8 as i16;

            ln!(self, 0x0c16, "mov     dx, ax");
            ln!(self, 0x0c18, "push    di");
            ln!(self, 0x0c19, "call    sub_1C661");
            self.sub_1c661(dx_extended, bx_extended, i);

            ln!(self, 0x0c1c, "pop     di");
            ln!(
                self,
                0x0c1d,
                "cmp     word ptr [di], 320 ; particle.rect.x0",
                MemRef::Dw(0x3cc0 + i * 0x11)
            );
            ln!(self, 0x0c21, "jnb     short loc_10C2F");

            let mut should_remove = (self.particle(i).rect.x0 as u16) >= 320;
            if !should_remove {
                ln!(self, 0x0c23, "xor     ax, ax");
                ln!(
                    self,
                    0x0c25,
                    "cmp     [di+4], ax ; particle.rect.x1",
                    MemRef::Dw(0x3cc0 + i * 0x11 + 4)
                );
                ln!(self, 0x0c28, "js      short loc_10C2F");
                should_remove = self.particle(i).rect.x1 < 0;
            }

            if !should_remove {
                ln!(
                    self,
                    0x0c2a,
                    "cmp     [di+6], ax ; particle.rect.y1",
                    MemRef::Dw(0x3cc0 + i * 0x11 + 6)
                );
                ln!(self, 0x0c2d, "jns     short loc_10C34");
                should_remove = self.particle(i).rect.y1 < 0;
            }

            if should_remove {
                ln!(self, 0x0c2f, "push    di");
                ln!(self, 0x0c30, "call    _sub_1C58A_particles_remove_particle");
                self.sub_1c58a_particles_remove_particle(i);
                ln!(self, 0x0c33, "pop     di");
            }

            ln!(self, 0x0c34, "pop     cx");
            ln!(self, 0x0c35, "add     di, 11h ; sizeof(Particle) = 17");
            i += 1;

            ln!(self, 0x0c38, "loop    loc_10BC6");
        }

        ln!(self, 0x0c3a, "locret_10C3A: ret");
    }

    fn sub_10c3b(&mut self) {
        println!("===== 0x0c3b");
        ln!(self, 0x0c3b, "dec     byte ptr [si+1h]");
        ln!(self, 0x0c3e, "jns     short loc_10C79");
        let mut ax = 0;
        self.night_attack_data.timer1 = self.night_attack_data.timer1.wrapping_sub(1);
        if self.night_attack_data.timer1 < 0 {
            ln!(self, 0x0c40, "test    word ptr [0h], 3", MemRef::Dw(0x0000));
            ln!(self, 0x0c46, "jnz     short loc_10C58");
            if (self.word_1f4b0_rand_bits & 3) == 0 {
                ln!(self, 0x0c48, "mov     byte ptr [si+7h], 0Bh");
                self.night_attack_data.timer7 = 0x0B;
                ln!(
                    self,
                    0x0c4c,
                    "test    word ptr [0h], 0Ch",
                    MemRef::Dw(0x0000)
                );
                ln!(self, 0x0c52, "jnz     short loc_10C58");
                if (self.word_1f4b0_rand_bits & 0x0C) == 0 {
                    ln!(self, 0x0c54, "mov     byte ptr [si+7h], 11h");
                    self.night_attack_data.timer7 = 0x11;
                }
            }

            // ln!(self, 0x0c58, "loc_10C58:");
            ln!(self, 0x0c58, "call    _sub_1E3CC_rand");
            ax = self.sub_1e3cc_rand();
            ln!(self, 0x0c5b, "cmp     byte_23BEA, 0");
            ln!(self, 0x0c60, "jz      short loc_10C64");
            if self.byte_23bea != 0 {
                ln!(self, 0x0c62, "and     al, 0EFh");
                ax &= 0xFFEF; // Clear bit 4 in al
            }

            ln!(self, 0x0c64, "mov     cx, ax");
            let mut cx = ax;
            ln!(self, 0x0c66, "mov     bx, 7");
            ln!(self, 0x0c69, "call    _sub_1E3B7_rand_masked");
            let masked_random = self.sub_1e3b7_rand_masked(7);
            ln!(self, 0x0c6c, "mov     [si+1h], al", MemRef::Db(0x4857));
            self.night_attack_data.timer1 = (masked_random & 0xFF) as i8;
            ln!(self, 0x0c6f, "cmp     al, 4");
            ln!(self, 0x0c71, "jb      short loc_10C76");
            if (masked_random & 0xFF) >= 4 {
                ln!(self, 0x0c73, "or      ch, 40h");
                cx |= 0x4000; // Set bit 6 in ch
            }

            // ln!(self, 0x0c76, "loc_10C76:");
            // ln!(self, 0x0c76, "mov     [si+2], cx");
            self.night_attack_data.unk2 = (cx & 0xFF) as u8;
            self.night_attack_data.unk3 = ((cx >> 8) & 0xFF) as u8;

            ln!(self, 0x0c76, "mov     [si+2], cx");
        }

        // ln!(self, 0x0c79, "loc_10C79:");
        ln!(self, 0x0c79, "mov     byte ptr [si], 8");
        self.night_attack_data.timer0 = 8;
        ln!(self, 0x0c7c, "mov     al, [si+2]");
        // ax = (ax & 0xff00) | (self.night_attack_data.unk2 as u16);
        let mut ax = self.night_attack_data.unk2 as u16;
        println!("                 ax = {ax:04x}");
        let di = ax;
        ln!(self, 0x0c7f, "mov     di, ax");
        ln!(self, 0x0c81, "mov     bl, [si+3]");
        let bl = self.night_attack_data.unk3;

        ln!(self, 0x0c84, "and     ax, 10h");
        ln!(self, 0x0c87, "jnz     short loc_10C98");
        ax &= 0x10;
        if ax == 0 {
            ln!(self, 0x0c89, "and     bx, 6");
            let bx_masked: usize = (bl & 6) as usize;
            ln!(self, 0x0c8c, "mov     si, word_20A52[bx]");
            let si = self.word_20a52[bx_masked >> 1];
            println!("si = {:04x}", si);

            ln!(self, 0x0c90, "add     bx, bx");
            let bx_doubled = bx_masked * 2;
            println!("              bx = {bx_doubled:04x}");

            ln!(self, 0x0c92, "add     ax, bx");
            ax += bx_doubled as u16;
            println!("              ax = {ax:04x}");

            ln!(self, 0x0c94, "add     al, 4");
            ax = (ax & 0xFF00) | (((ax & 0xFF) + 4) & 0xFF);
            println!("              ax = {ax:04x}");

            ln!(self, 0x0c96, "jmp     short loc_10CD6");

            // Jump to final particle spawn
            self.spawn_particle_final(ax, di, si);
        } else {
            // ln!(self, 0x0c98, "loc_10C98:");
            ln!(self, 0x0c98, "mov     al, [si+3]");
            ln!(self, 0x0c9b, "mov     ah, al");
            let mut al = bl;
            let mut ah = bl;
            ln!(self, 0x0c9d, "and     ax, 0C03Fh");
            let mut ax = ((ah as u16) << 8) | (al as u16);
            ax &= 0xC03F;
            al = (ax & 0xFF) as u8;
            ah = ((ax >> 8) & 0xFF) as u8;

            ln!(self, 0x0ca0, "test    ah, 40h");
            ln!(self, 0x0ca3, "jz      short loc_10CCA");
            if (ah & 0x40) != 0 {
                ln!(self, 0x0ca5, "rol     word ptr [0h], 1", MemRef::Dw(0x0000));
                ln!(self, 0x0ca9, "jnb     short loc_10CCA");
                self.word_1f4b0_rand_bits = self.word_1f4b0_rand_bits.rotate_left(1);
                if (self.word_1f4b0_rand_bits & 1) != 0 {
                    ln!(self, 0x0cab, "mov     cl, 0Ah");
                    let mut cl: u8 = 0x0A;
                    ln!(self, 0x0cad, "or      ah, ah");
                    ln!(self, 0x0caf, "jns     short loc_10CB3");
                    if (ah & 0x80) != 0 {
                        ln!(self, 0x0cb1, "neg     cl");
                        cl = cl.wrapping_neg();
                    }

                    // ln!(self, 0x0cb3, "loc_10CB3:");
                    ln!(self, 0x0cb3, "add     al, cl");
                    al = al.wrapping_add(cl);
                    ln!(self, 0x0cb5, "jns     short loc_10CBC");
                    if (al & 0x80) != 0 {
                        ln!(self, 0x0cb7, "xor     ah, 80h");
                        ah ^= 0x80;
                        ln!(self, 0x0cba, "xor     al, al");
                        al = 0;
                    }

                    // ln!(self, 0x0cbc, "loc_10CBC:");
                    ln!(self, 0x0cbc, "cmp     al, 40h");
                    ln!(self, 0x0cbe, "jb      short loc_10CC5");
                    if al >= 0x40 {
                        ln!(self, 0x0cc0, "mov     al, 3Fh");
                        al = 0x3F;
                        ln!(self, 0x0cc2, "xor     ah, 80h");
                        ah ^= 0x80;
                    }

                    // ln!(self, 0x0cc5, "loc_10CC5:");
                    ln!(self, 0x0cc5, "or      ah, al");
                    ah |= al;
                    ln!(self, 0x0cc7, "mov     [si+3], ah");
                    self.night_attack_data.unk3 = ah;
                }
            }

            // ln!(self, 0x0cca, "loc_10CCA:");
            ln!(self, 0x0cca, "add     al, 0E0h");
            al = al.wrapping_add(0xE0);
            ln!(self, 0x0ccc, "call    sub_15198");
            // println!("            -> al = {:02x}", al);
            let (bx, dx) = self.sub_15198(al);
            // println!("            -> {:04x}, {:04x}", result.0, result.1);
            ln!(self, 0x0ccf, "mov     dh, bl");
            ln!(self, 0x0cd1, "mov     si, dx");
            // let si = ((night_attack_data.unk3 as u16) << 8) | (result.0 & 0xFF);
            let dl = (dx as u16) & 0x00ff;
            let bl = bx & 0x00ff;
            let si = (bl << 8) | dl;
            ln!(self, 0x0cd3, "mov     ax, 14h");
            let ax = 0x14;

            // Fall through to final particle spawn
            self.spawn_particle_final(ax, di, si);
        }
    }

    fn sub_15198(&mut self, value: u8) -> (u16, i16) {
        ln!(self, 0x5198, "mov     bx, ax");
        let mut bx = value as u16;
        ln!(self, 0x519a, "add     bl, 20h");
        let mut bl = (bx as u8).wrapping_add(0x20);
        ln!(self, 0x519d, "mov     bh, bl");
        let mut bh = bl;
        ln!(self, 0x519f, "and     bh, 7fh");
        bh &= 0x7F;

        ln!(self, 0x51a2, "cmp     bh, 40h");
        ln!(self, 0x51a5, "jb      short loc_151BA");
        let (bx, dx) = if bh < 0x40 {
            // ln!(self, 0x51ba, "loc_151BA:");
            ln!(self, 0x51ba, "or      bl, bl");
            ln!(self, 0x51bc, "mov     bx, 0FFE0h");
            bx = 0xFFE0;
            ln!(self, 0x51bf, "jns     short loc_151C7");
            if (bl as i8) >= 0 {
                // ln!(self, 0x51c7, "loc_151C7:");
                ln!(self, 0x51c7, "cbw");
                let dx = (value as i8) as i16;
                ln!(self, 0x51c8, "mov     dx, ax");
                ln!(self, 0x51ca, "ret");
                (bx, dx)
            } else {
                ln!(self, 0x51c1, "sub     al, 80h");
                let mut al = value.wrapping_sub(0x80);
                ln!(self, 0x51c3, "neg     al");
                al = al.wrapping_neg();
                ln!(self, 0x51c5, "neg     bx");
                bx = (-(bx as i16)) as u16;
                ln!(self, 0x51c7, "cbw");
                let dx = (al as i8) as i16;
                ln!(self, 0x51c8, "mov     dx, ax");
                ln!(self, 0x51ca, "ret");
                (bx, dx)
            }
        } else {
            ln!(self, 0x51a7, "mov     dx, 20h");
            let mut dx = 0x20i16;
            ln!(self, 0x51aa, "sub     al, 40h");
            let mut al = value.wrapping_sub(0x40);
            ln!(self, 0x51ac, "or      bl, bl");
            ln!(self, 0x51ae, "jns     short loc_151B6");
            if (bl as i8) < 0 {
                ln!(self, 0x51b0, "neg     dx");
                dx = -dx;
                ln!(self, 0x51b2, "sub     al, 80h");
                al = al.wrapping_sub(0x80);
                ln!(self, 0x51b4, "neg     al");
                al = al.wrapping_neg();
            }

            // ln!(self, 0x51b6, "loc_151B6:");
            ln!(self, 0x51b6, "cbw");
            ln!(self, 0x51b7, "mov     bx, ax");
            bx = (al as i8) as i16 as u16;
            ln!(self, 0x51b9, "ret");
            (bx, dx)
        };

        // println!("=====");
        // println!("            bx = {bx:04x} dx = {dx:04x}");
        (bx, dx)
    }

    fn spawn_particle_final(&mut self, ax: u16, di: u16, si: u16) {
        // ln!(self, 0x0cd6, "loc_10CD6:");
        ln!(self, 0x0cd6, "and     di, 0Ch");
        let di_masked = di & 0x0C;
        // println!("di = {:04x}, di_masked = {:02x}", di, di_masked);
        ln!(
            self,
            0x0cd9,
            "mov     bx, word_20A44[di]",
            MemRef::Dw(0x1592 + di_masked + 2)
        );
        let bx = self.word_20a42[((di_masked >> 1) + 1) as usize]; // word array
        // println!("bx = {bx:04x}");
        ln!(
            self,
            0x0cdd,
            "mov     dx, word_20A42[di]",
            MemRef::Dw(0x1592 + di_masked)
        );
        let dx = self.word_20a42[(di_masked >> 1) as usize]; // word array
        // println!("dx = {dx:04x}");
        ln!(self, 0x0ce1, "call    _sub_1C60B_particles_spawn_particle");
        // println!("\nsi = {si:04x}\n");
        if self
            .sub_1c60b_particles_spawn_particle(ax, dx, bx, si)
            .is_some()
        {
            ln!(self, 0x0ce4, "mov     word ptr [di+0dh], 0");
            // Set data0 and data1 to 0 for the newly created particle
            let last_particle_idx = (self.word_2316e_particle_count - 1) as usize;
            self.unk_23170_particles[last_particle_idx].data0 = 0;
            self.unk_23170_particles[last_particle_idx].data1 = 0;
        }
        ln!(self, 0x0ce9, "ret");
    }

    fn sub_10cea_word(&self, bx: &mut u16, dx: &mut u16) {
        let mut bl = (*bx & 0xFF) as i8;
        let mut bh = ((*bx >> 8) & 0xFF) as i8;
        let mut dl = (*dx & 0xFF) as i8;
        let mut dh = ((*dx >> 8) & 0xFF) as i8;

        // println!(" sub_10cea_word +++++  bl = {bl}, bh = {bh}, dl = {dl}, dh = {dh}");

        self.sub_10cea(&mut bl, &mut bh, &mut dl, &mut dh);

        // println!(" sub_10cea_word +++++  bl = {bl}, bh = {bh}, dl = {dl}, dh = {dh}");

        *bx = ((bh as u8 as u16) << 8) | (bl as u8 as u16);
        *dx = ((dh as u8 as u16) << 8) | (dl as u8 as u16);

        // println!("            bx = {bx:04x}");
        // println!("            dx = {dx:04x}");
    }

    fn sub_10cea(&self, bl: &mut i8, bh: &mut i8, dl: &mut i8, dh: &mut i8) {
        // ln!(self, 0x0cea, " xor     ax, ax");
        // ln!(self, 0x0cec, " mov     al, dl");
        let mut al = *dl;

        ln!(self, 0x0cea, "xor     ax, ax");
        ln!(self, 0x0cec, "mov     al, dl");
        ln!(self, 0x0cee, "or      al, al");
        ln!(self, 0x0cf0, "js      0d05h");

        if al < 0 {
            ln!(self, 0x0d05, "neg     al");
            al = al.wrapping_neg();

            ln!(self, 0x0d07, "call    0cf2h");
            self.sub_10cf2(bl, bh, dl, dh, al);

            *dh = dh.wrapping_neg();
            ln!(self, 0x0d0a, "neg     dh");
            ln!(self, 0x0d0c, "ret");
        } else {
            self.sub_10cf2(bl, bh, dl, dh, al);
        }
    }

    fn sub_10cf2(&self, bl: &mut i8, bh: &mut i8, dl: &mut i8, dh: &mut i8, mut al: i8) {
        // 8-bit wrapping add
        al = al.wrapping_add(*bl);
        ln!(self, 0x0cf2, "add     al, bl; al = {al:02x}");

        // Build ax with ah=0, rotate, then extract
        let mut ax = (al as u8) as u16;
        ax = ax.rotate_right(5);
        ln!(self, 0x0cf4, "mov     cl, 5h");
        ln!(self, 0x0cf6, "ror     ax, cl; ax = {ax:04x}");

        let al2 = ax as u8;
        let ah2 = (ax >> 8) as u8;

        ln!(self, 0x0cf8, "mov     cl, 3h");
        ln!(self, 0x0cfa, "shr     ah, cl; ah = {ah2:02x}");

        *bl = (ah2 >> 3) as i8; // logical shift
        *dl = al2 as i8;

        // println!("             bl = {bl:02x}, bh = {bh:02x}, dl = {dl:02x}, dh = {dh:02x}");

        ln!(self, 0x0cfc, "mov     bl, ah; bl = {bl:02x}");
        ln!(self, 0x0cfe, "mov     dl, al; dl = {dl:02x}");

        // println!("             bl = {bl:02x}, bh = {bh:02x}, dl = {dl:02x}, dh = {dh:02x}");

        core::mem::swap(bl, bh);
        core::mem::swap(dl, dh);

        // println!("             bl = {bl:02x}, bh = {bh:02x}, dl = {dl:02x}, dh = {dh:02x}");

        ln!(self, 0x0d00, "xchg    bl, bh; bl = {bl:02x}, bh = {bh:02x}");
        // println!("             bl = {bl:02x}, bh = {bh:02x}, dl = {dl:02x}, dh = {dh:02x}");
        ln!(self, 0x0d02, "xchg    dl, dh; dl = {dl:02x}, dh = {dh:02x}");
        // println!("             bl = {bl:02x}, bh = {bh:02x}, dl = {dl:02x}, dh = {dh:02x}");
        ln!(self, 0x0d04, "ret");
    }

    fn sub_10d0d(&mut self, flag: bool) {
        ln!(self, 0x0d0d, "mov     al, bl");
        let al = self.night_attack_data.timer7;

        ln!(self, 0x0d0f, "mov     bx, 384");
        let bx = 384u16;
        ln!(self, 0x0d12, "mov     cx, 84");
        let cx = 84u16;
        ln!(self, 0x0d15, "mov     dl, 55");
        let mut dl = 55u8;

        ln!(self, 0x0d17, "jz      short loc_10D23");
        if !flag {
            ln!(self, 0x0d19, "dec     dx");
            dl -= 1;
            ln!(self, 0x0d1a, "cmp     al, 0ah");
            ln!(self, 0x0d1c, "jz      short loc_10d23");

            if self.night_attack_data.timer7 != 10 {
                ln!(
                    self,
                    0x0d1e,
                    "call    _ptr_22d65_gfx_vtable_func_39_transition_palette"
                );
                // self.gfx_vtable_func_39_transition_palette(al, bx, cx, dl);

                for i in 128..128 + 0x1c {
                    let divisor = self.night_attack_data.timer7.max(1) as i16;

                    let r = (self.pal_2bf.get(i).0 as i16 - self.pal_5bf.get(i).0 as i16) / divisor
                        + self.pal_5bf.get(i).0 as i16;
                    let g = (self.pal_2bf.get(i).1 as i16 - self.pal_5bf.get(i).1 as i16) / divisor
                        + self.pal_5bf.get(i).1 as i16;
                    let b = (self.pal_2bf.get(i).2 as i16 - self.pal_5bf.get(i).2 as i16) / divisor
                        + self.pal_5bf.get(i).2 as i16;

                    let rgb = (r as u8, g as u8, b as u8);

                    println!(
                        "{:02x} - {:02x} / {divisor} -> {:02x}",
                        self.pal_2bf.get(i).0,
                        self.pal_5bf.get(i).0,
                        rgb.0
                    );
                    println!(
                        "{:02x} - {:02x} / {divisor} -> {:02x}",
                        self.pal_2bf.get(i).1,
                        self.pal_5bf.get(i).1,
                        rgb.1
                    );
                    println!(
                        "{:02x} - {:02x} / {divisor} -> {:02x}",
                        self.pal_2bf.get(i).2,
                        self.pal_5bf.get(i).2,
                        rgb.2
                    );

                    self.pal_5bf.set(i, rgb);
                    self.screen_pal.set(i, rgb);
                }

                ln!(self, 0x0d22, "retn");
                return;
            }
        }

        ln!(self, 0x0d23, "call    _sub_1C13B_open_onmap_resource");
        self.sub_1c13b_open_onmap_resource();
        let ax = dl as u16;
        ln!(self, 0x0d26, "mov     al, dl");
        ln!(self, 0x0d28, &format!("xor     ah, ah; ax = {ax:04x}"));

        let si = self.sprite_sheet.get_resource(ax as usize).unwrap();
        ln!(
            self,
            0x0d2a,
            "call    _sub_1C1F4_get_subresource_ax_pointer_to_dssi"
        );
        let dsdx = &si[6..];

        ln!(self, 0x0d2d, "lea     dx, [si+6]");
        ln!(
            self,
            0x0d30,
            "call    _ptr_22D65_gfx_vtable_func_02_set_pal_2"
        );

        for i in 0..28 {
            let rgb = (dsdx[3 * i], dsdx[3 * i + 1], dsdx[3 * i + 2]);
            self.pal_5bf.set(128 + i, rgb);
            self.screen_pal.set(128 + i, rgb);
        }

        ln!(self, 0x0d34, "call    _sub_1C0F4_update_screen_palette");

        ln!(self, 0x0d37, "mov     ax, 35h"); // 53
        ln!(
            self,
            0x0d3a,
            "call    _sub_1C1F4_get_subresource_ax_pointer_to_dssi"
        );

        let si = self.sprite_sheet.get_resource(53).unwrap();

        ln!(self, 0x0d3d, "lea     dx, [si+6]");

        let dsdx = &si[6..];

        ln!(
            self,
            0x0d40,
            "call    _ptr_22D65_gfx_vtable_func_38_set_pal_1"
        );

        for i in 0..28 {
            let rgb = (dsdx[3 * i], dsdx[3 * i + 1], dsdx[3 * i + 2]);
            self.pal_2bf.set(128 + i, rgb);
        }

        ln!(self, 0x0d44, "retn");
    }

    fn sub_1c202(&self, sprite_id: u16, center_x: &mut i16, center_y: &mut i16) {
        ln!(self, 0xc202, "push    ax");
        ln!(self, 0xc203, "push    si");
        ln!(self, 0xc204, "call    0c1f4h");

        println!("                 ax = {:04x}", sprite_id);

        ln!(self, 0xc1f4, "push    bx");
        ln!(self, 0xc1f5, "les     si, [0dbb0h]");
        ln!(self, 0xc1f9, "mov     bx, ax");
        ln!(self, 0xc1fb, "shl     bx, 1");
        ln!(self, 0xc1fd, "add     si, es:[bx+si]");
        ln!(self, 0xc200, "pop     bx");
        ln!(self, 0xc201, "ret");
        // println!("=====");

        let Some(sprite) = self.sprite_sheet.get_sprite(sprite_id as usize) else {
            return;
        };

        let width = sprite.width();
        let height = sprite.height();

        ln!(self, 0xc207, "es:lodsw");
        ln!(self, 0xc209, "and     ah, 0fh");
        ln!(self, 0xc20c, "shr     ax, 1");
        ln!(self, 0xc20e, "sub     dx, ax");
        ln!(self, 0xc210, "es:lodsb");
        ln!(self, 0xc212, "shr     al, 1");
        ln!(self, 0xc214, "cbw");
        ln!(self, 0xc215, "sub     bx, ax");

        *center_x = center_x.saturating_sub_unsigned(width / 2);
        *center_y = center_y.saturating_sub_unsigned(height / 2);

        ln!(self, 0xc217, "pop     si");
        ln!(self, 0xc218, "pop     ax");
        ln!(self, 0xc219, "ret");
        // println!("=====");
    }

    fn sub_1c60b_particles_spawn_particle(
        &mut self,
        sprite_id: u16,
        center_x: i16,
        center_y: i16,
        subtype: u16,
    ) -> Option<&mut Particle> {
        println!("===== 0xc60b_spawn_particle");
        println!(
            "##### sub_1c60b {} {} {} {:04x}",
            sprite_id, center_x, center_y, subtype
        );
        // dbg!(sprite_id, center_x, center_y, subtype);

        ln!(self, 0xc60b, "push    ax");
        ln!(self, 0xc60c, "call    0c13bh");
        self.sub_1c13b_open_onmap_resource();

        let mut x0 = center_x;
        let mut y0 = center_y;

        ln!(self, 0xc60f, "pop     ax");
        ln!(self, 0xc610, "call    0c202h");

        self.sub_1c202(sprite_id, &mut x0, &mut y0);

        ln!(self, 0xc613, "push    si");
        ln!(self, 0xc614, "mov     di, 3cbeh");
        ln!(self, 0xc617, "mov     bp, ax");
        ln!(self, 0xc619, "mov     ax, [di]");
        ln!(self, 0xc61b, "inc     word ptr [di]");
        ln!(self, 0xc61d, "mov     ah, 11h");
        ln!(self, 0xc61f, "mul     ah");
        ln!(self, 0xc621, "xchg    bp, ax");
        ln!(self, 0xc622, "lea     di, [bp+di+2h]");

        let di = 0x3cbe + 2 + 0x11 * self.word_2316e_particle_count;

        self.unk_23170_particles[self.word_2316e_particle_count as usize]
            .rect
            .x0 = x0;
        ln!(self, 0xc625, "mov     [di], dx", MemRef::Dw(di));

        self.unk_23170_particles[self.word_2316e_particle_count as usize]
            .rect
            .y0 = y0;
        ln!(self, 0xc627, "mov     [di+2h], bx", MemRef::Dw(di + 2));

        self.unk_23170_particles[self.word_2316e_particle_count as usize].sprite_id = sprite_id;
        ln!(self, 0xc62a, "mov     [di+8h], ax", MemRef::Dw(di + 8));

        self.unk_23170_particles[self.word_2316e_particle_count as usize].subtype = subtype;
        ln!(self, 0xc62d, "mov     [di+0ah], si", MemRef::Dw(di + 0xa));

        ln!(
            self,
            0xc630,
            "mov     byte ptr [di+0ch], 0h",
            MemRef::Dw(di + 0xc)
        );
        self.unk_23170_particles[self.word_2316e_particle_count as usize].flags = 0;

        ln!(self, 0xc634, "call    0c1f4h");

        let sprite = self.sprite_sheet.get_sprite(sprite_id as usize).unwrap();

        ln!(self, 0xc1f4, "push    bx");
        ln!(self, 0xc1f5, "les     si, [0dbb0h]");
        ln!(self, 0xc1f9, "mov     bx, ax");
        ln!(self, 0xc1fb, "shl     bx, 1");
        ln!(self, 0xc1fd, "add     si, es:[bx+si]");
        ln!(self, 0xc200, "pop     bx");
        ln!(self, 0xc201, "ret");
        // println!("=====");

        let width = sprite.width();
        let height = sprite.height();

        ln!(self, 0xc637, "es:lodsw");
        ln!(self, 0xc639, "and     ah, 0fh");
        ln!(self, 0xc63c, "add     dx, ax");
        ln!(self, 0xc63e, "add     bl, es:[si]");
        ln!(self, 0xc641, "adc     bh, 0h");
        ln!(self, 0xc644, "mov     [di+4h], dx");
        ln!(self, 0xc647, "mov     [di+6h], bx");

        self.unk_23170_particles[self.word_2316e_particle_count as usize]
            .rect
            .x1 = x0.saturating_add_unsigned(width);
        self.unk_23170_particles[self.word_2316e_particle_count as usize]
            .rect
            .y1 = y0.saturating_add_unsigned(height);

        // self.unk_23170_particles[self.word_2316e_particle_count as usize] = Particle {
        //     rect: Rect { x0, y0, x1: x0 + width, y1: y0 + height },
        //     sprite_id,
        //     subtype,
        //     flags: 0,
        //     data0: 0,
        //     data1: 0,
        //     data2: 0,
        //     data3: 0,
        // };
        self.word_2316e_particle_count += 1;

        ln!(self, 0xc64a, "pop     si");
        ln!(self, 0xc64b, "ret");

        Some(&mut self.unk_23170_particles[self.word_2316e_particle_count as usize])
    }

    fn dump_particles(&self, frame_number: usize) {
        println!();
        println!("===== particles: {}", self.word_2316e_particle_count);

        for i in 0..self.word_2316e_particle_count {
            let particle = &self.unk_23170_particles[i as usize];
            println!(
                "particle_list: frame: {:5} {i:-2}: [{:4} {:4} {:4} {:4}] id={:02x} type={:04x} flags={:02x} data={:02x} {:02x} {:02x} {:02x}",
                frame_number,
                particle.rect.x0,
                particle.rect.y0,
                particle.rect.x1,
                particle.rect.y1,
                particle.sprite_id,
                particle.subtype,
                particle.flags,
                particle.data0,
                particle.data1,
                particle.data2,
                particle.data3
            );
        }
        println!();
    }

    fn sub_1c661(&mut self, dx: i16, bx: i16, particle_index: u16) {
        let particle_index = particle_index as usize;
        println!("===== 0xc661 dx = {dx:04x}, bx = {bx:04x}, di = {particle_index:04x}");
        ln!(self, 0xc661, "call    _sub_1C13B_open_onmap_resource");
        self.sub_1c13b_open_onmap_resource();

        ln!(self, 0xc664, "mov     si, di");
        ln!(self, 0xc666, "sub     sp, 8");
        ln!(self, 0xc669, "mov     di, sp");
        ln!(self, 0xc66b, "push    ds");
        ln!(self, 0xc66c, "pop     es");
        ln!(self, 0xc66d, "movsw");
        ln!(self, 0xc66e, "movsw");
        ln!(self, 0xc66f, "movsw");
        ln!(self, 0xc670, "movsw");
        // Copy particle rect to stack (8 bytes = 4 words)
        let mut temp_rect = self.unk_23170_particles[particle_index].rect;

        ln!(self, 0xc671, "sub     si, 8h");
        ln!(self, 0xc674, "sub     di, 8h");
        // Reset pointers to start of rects

        self.unk_23170_particles[particle_index].rect.x0 = self.unk_23170_particles[particle_index]
            .rect
            .x0
            .wrapping_add(dx);
        ln!(
            self,
            0xc677,
            "add     [si], dx",
            MemRef::Dw((0x3cc0 + particle_index * 0x11) as u16)
        );

        self.unk_23170_particles[particle_index].rect.y0 = self.unk_23170_particles[particle_index]
            .rect
            .y0
            .wrapping_add(bx);
        ln!(
            self,
            0xc679,
            "add     [si+2h], bx",
            MemRef::Dw((0x3cc0 + particle_index * 0x11 + 2) as u16)
        );

        self.unk_23170_particles[particle_index].rect.x1 = self.unk_23170_particles[particle_index]
            .rect
            .x1
            .wrapping_add(dx);
        ln!(
            self,
            0xc67c,
            "add     [si+4h], dx",
            MemRef::Dw((0x3cc0 + particle_index * 0x11 + 4) as u16)
        );

        self.unk_23170_particles[particle_index].rect.y1 = self.unk_23170_particles[particle_index]
            .rect
            .y1
            .wrapping_add(bx);
        ln!(
            self,
            0xc67f,
            "add     [si+6h], bx",
            MemRef::Dw((0x3cc0 + particle_index * 0x11 + 6) as u16)
        );

        ln!(self, 0xc682, "or      dx, dx");
        ln!(self, 0xc684, "js      short loc_1C68E");
        if dx >= 0 {
            ln!(self, 0xc686, "mov     ax, [si+4h]");
            ln!(self, 0xc689, "mov     [di+4], ax");
            ln!(self, 0xc68c, "jmp     0c692h");
            temp_rect.x1 = self.unk_23170_particles[particle_index].rect.x1;
        } else {
            ln!(self, 0xc68e, "mov     ax, [si]");
            ln!(self, 0xc690, "mov     [di], ax");
            temp_rect.x0 = self.unk_23170_particles[particle_index].rect.x0;
        }

        ln!(self, 0xc692, "or      bx, bx");
        ln!(self, 0xc694, "js      short loc_1C69E");
        if bx >= 0 {
            ln!(self, 0xc696, "mov     ax, [si+6h]");
            ln!(self, 0xc699, "mov     [di+6], ax");
            temp_rect.y1 = self.unk_23170_particles[particle_index].rect.y1;
            ln!(self, 0xc69c, "jmp     short loc_1C6A4");
        } else {
            // ln!(self, 0xc69e, "loc_1C69E:");
            ln!(self, 0xc69e, "mov     ax, [si+2]");
            ln!(self, 0xc6a1, "mov     [di+2], ax");
            temp_rect.y0 = self.unk_23170_particles[particle_index].rect.y0;
        }

        // ln!(self, 0xc6a4, "loc_1C6A4:");
        ln!(self, 0xc6a4, "mov     si, di");
        ln!(
            self,
            0xc6a6,
            "call    _sub_1C6AD_particles_update_dirty_rect"
        );
        self.sub_1c6ad_particles_update_dirty_rect(&temp_rect);

        ln!(self, 0xc6a9, "add     sp, 8");
        ln!(self, 0xc6ac, "ret");
        // Stack cleanup (temp_rect goes out of scope)
    }

    fn sub_1c13b_open_onmap_resource(&self) {
        ln!(self, 0xc13b, "mov     ax, 2bh");
        ln!(self, 0xc13e, "or      ax, ax");
        ln!(self, 0xc140, "js      0c1a9h");
        ln!(self, 0xc142, "push    bx");
        ln!(self, 0xc143, "mov     bx, ax");
        ln!(self, 0xc145, "xchg    bx, [2784h] ; current bank id");
        ln!(self, 0xc149, "cmp     ax, bx");
        ln!(self, 0xc14b, "jz      0c1a8h");
        ln!(self, 0xc1a8, "pop     bx");
        ln!(self, 0xc1a9, "ret");
        // println!("=====");
    }

    fn sub_1c58a_particles_remove_particle(&mut self, index: u16) {
        println!("===== 0xc58a_remove_particle");
        ln!(self, 0xc58a, "call    0c13bh");
        self.sub_1c13b_open_onmap_resource();

        ln!(self, 0xc58d, "mov     si, 3cbeh");
        ln!(self, 0xc590, "lodsw");
        ln!(self, 0xc591, "or      ax, ax");
        ln!(self, 0xc593, "jz      0c5ceh");

        if self.word_2316e_particle_count != 0 {
            ln!(self, 0xc595, "mov     ah, 11h");
            ln!(self, 0xc597, "mul     ah");
            ln!(self, 0xc599, "add     si, ax");
            ln!(self, 0xc59b, "cmp     di, si");
            ln!(self, 0xc59d, "jnb     0c5ceh");
            if index < self.word_2316e_particle_count {
                self.unk_23170_particles[index as usize].flags |= 0x80;
                ln!(
                    self,
                    0xc59f,
                    "or      byte ptr [di+0ch], 80h",
                    MemRef::Db(0x3cc0 + 0x11 * index + 0xc)
                );

                ln!(self, 0xc5a3, "push    si");
                ln!(self, 0xc5a4, "push    di");
                ln!(self, 0xc5a5, "mov     si, di");
                ln!(self, 0xc5a7, "call    0c6adh");

                let rect = self.unk_23170_particles[index as usize].rect;
                self.sub_1c6ad_particles_update_dirty_rect(&rect);

                ln!(self, 0xc5aa, "pop     di");
                ln!(self, 0xc5ab, "pop     cx");
                ln!(self, 0xc5ac, "push    di");
                ln!(self, 0xc5ad, "lea     si, [di+11h]");
                ln!(self, 0xc5b0, "sub     cx, si");
                ln!(self, 0xc5b2, "jz      0c5b8h");

                if index < self.word_2316e_particle_count - 1 {
                    ln!(self, 0xc5b4, "push    ds");
                    ln!(self, 0xc5b5, "pop     es");
                    ln!(self, 0xc5b6, "rep movsb");
                    for i in index..self.word_2316e_particle_count - 1 {
                        self.unk_23170_particles[i as usize] =
                            self.unk_23170_particles[i as usize + 1];
                    }
                }

                self.word_2316e_particle_count -= 1;
                ln!(self, 0xc5b8, "dec     word ptr [3cbeh]", MemRef::Dw(0x3cbe));

                ln!(self, 0xc5bc, "pop     di");
                ln!(self, 0xc5bd, "mov     si, 4752h");
                ln!(self, 0xc5c0, "mov     cx, 2h");
                for i in 0..2 {
                    ln!(self, 0xc5c3, "lodsw");
                    ln!(self, 0xc5c4, "cmp     ax, di");
                    ln!(self, 0xc5c6, "jb      0c5cch");
                    if false {
                        // TODO
                        ln!(self, 0xc5c8, "sub     word ptr [si-2], 11h");
                    }
                    ln!(self, 0xc5cc, "loop    0c5c3h");
                }
            }
        }
        ln!(self, 0xc5ce, "ret");
    }

    fn gfx_vtable_func_39_transition_palette(&mut self, al: u8, bx: u16, cx: u16, dl: u8) {
        let speed = u16::max(al as u16, 1);
        let offset = bx;
        let count = cx;

        for i in 0..count {
            // let d = src[i] - dst[i];
            // dst[i] += d / speed;
        }
    }

    fn sub_1c6ad_particles_update_dirty_rect(&mut self, mut dirty_rect: &Rect) {
        println!("===== 0xc6ad_update_dirty_rect");
        // ln!(self, 0xc6ad, "call    0c13bh");
        // self.sub_1c13b_open_onmap_resource();

        let screen_bounds = Rect {
            x0: 0,
            y0: 0,
            x1: 320,
            y1: 200,
        };

        let rect = dirty_rect.clip(&screen_bounds);

        for y in rect.y0 as u16..rect.y1 as u16 {
            for x in rect.x0 as u16..rect.x1 as u16 {
                let c = self
                    .bg_framebuffer
                    .get_pixel(x as usize, (y + GLOBAL_Y_OFFSET) as usize);
                self.framebuffer
                    .put_pixel(x as usize, (y + GLOBAL_Y_OFFSET) as usize, c);
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
                y0: GLOBAL_Y_OFFSET as i16,
                x1: 320,
                y1: 200,
            };

            self.sprite_sheet.draw_sprite_clipped(
                particle.sprite_id as usize,
                particle.rect.x0,
                particle.rect.y0 + GLOBAL_Y_OFFSET as i16,
                clip_rect,
                self.framebuffer,
            );
        }

        // ln!(self, 0xc7d3, "ret");
    }
}
