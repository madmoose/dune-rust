use std::io::Cursor;

use bytes_ext::ReadBytesExt;
use framebuffer::Framebuffer;
use sprite::SpriteSheet;

#[derive(Debug)]
pub struct Lipsync {
    pub rect: (u16, u16, u16, u16),
    pub image_groups: Vec<Vec<Image>>,
    pub animations: Vec<Animation>,
}

#[derive(Debug, Default)]
pub struct Animation {
    pub frames: Vec<Frame>,
}

#[derive(Debug, Default)]
pub struct Frame {
    pub image_groups: Vec<u8>,
}

#[derive(Debug)]
pub struct Image {
    pub id: u8,
    pub x: u8,
    pub y: u8,
}

impl Lipsync {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut r = Cursor::new(bytes);

        let _unk0 = r.read_le_u16().unwrap();
        let _size = r.read_le_u16().unwrap();

        let x0 = r.read_le_u16().unwrap();
        let y0 = r.read_le_u16().unwrap();
        let x1 = r.read_le_u16().unwrap();
        let y1 = r.read_le_u16().unwrap();

        let rect = (x0, y0, x1, y1);

        let image_groups_len = r.read_le_u16().unwrap();

        let image_groups_toc = r.position();
        let image_groups_offset_0 = r.read_le_u16().unwrap();
        let image_groups_entry_count = image_groups_offset_0 as usize / 2;

        let mut image_groups = Vec::with_capacity(image_groups_entry_count);

        for n in 0..image_groups_entry_count {
            r.set_position(image_groups_toc + 2 * n as u64);
            let begin = r.read_le_u16().unwrap() as u64;
            r.set_position(image_groups_toc + begin);

            let mut images = Vec::new();
            loop {
                let id = r.read_u8().unwrap();
                if id == 0 {
                    break;
                }
                let x = r.read_u8().unwrap();
                let y = r.read_u8().unwrap();

                images.push(Image { id, x, y });
            }
            image_groups.push(images);
        }

        let animations_toc = image_groups_toc + image_groups_len as u64 - 2;
        r.set_position(animations_toc);

        let animations_offset_0: u16 = r.read_le_u16().unwrap();
        let animations_entry_count = animations_offset_0 as usize / 2;

        let mut animations = Vec::with_capacity(animations_entry_count);
        for n in 0..animations_entry_count {
            r.set_position(animations_toc + 2 * n as u64);
            let begin = r.read_le_u16().unwrap() as u64;
            r.set_position(animations_toc + begin);

            let mut animation = Animation::default();
            let mut frame = Frame::default();

            loop {
                let image_group_idx = r.read_u8().unwrap();
                if image_group_idx == 0 {
                    animation.frames.push(frame);
                    frame = Frame::default();
                    continue;
                } else if image_group_idx == 0xFF {
                    break;
                }
                assert!(image_group_idx != 1);
                frame.image_groups.push(image_group_idx - 2);
            }

            frame.image_groups.shrink_to_fit();
            animations.push(animation);
        }

        Self {
            rect,
            image_groups,
            animations,
        }
    }

    pub fn animation_count(&self) -> usize {
        self.animations.len()
    }

    pub fn animation_frame_count(&self, animation: usize) -> usize {
        self.animations[animation].frames.len()
    }

    pub fn draw_image_group(
        &self,
        framebuffer: &mut Framebuffer,
        sprite_sheet: &SpriteSheet,
        image_group: usize,
    ) {
        println!("\tDrawing image group {image_group:#04x}");

        let image_group = &self.image_groups[image_group];

        for image in image_group {
            println!(
                "\t\tDrawing image {:#04x}, ({:#04x}, {:#04x})",
                image.id, image.x, image.y
            );
            if let Some(sprite) = sprite_sheet.get_sprite(image.id as usize - 1) {
                sprite
                    .draw(
                        image.x as i16, /* + lipsync.rect.0 as usize */
                        image.y as i16, /* + lipsync.rect.1 as usize */
                        framebuffer,
                    )
                    .unwrap();
            }
        }
    }

    pub fn draw_animation_frame(
        &self,
        framebuffer: &mut Framebuffer,
        sprite_sheet: &SpriteSheet,
        animation: usize,
        frame: usize,
    ) {
        let animation = &self.animations[animation];
        let frame = &animation.frames[frame];

        for &image_group_idx in &frame.image_groups {
            let image_group = image_group_idx as usize;
            self.draw_image_group(framebuffer, sprite_sheet, image_group);
        }
    }

    pub fn display(&self) {
        println!("Lipsync data:");
        println!("  rect: {:?}", self.rect);
        for (i, image_group) in self.image_groups.iter().enumerate() {
            println!("  image_group {i:#04x}");
            for image in image_group {
                println!(
                    "    image: {:#04x} ({:#04x}, {:#04x})",
                    image.id, image.x, image.y
                );
            }
        }
        for (i, animation) in self.animations.iter().enumerate() {
            println!("  animation {i:#04x}");
            for (j, frame) in animation.frames.iter().enumerate() {
                print!("    frame {j:#04x}: ");
                for &image_group_idx in &frame.image_groups {
                    print!("{:#04x} ", image_group_idx + 2);
                }
                println!();
            }
        }
    }
}
