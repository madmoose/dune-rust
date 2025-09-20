use bin_read::BinRead;

#[derive(BinRead, Debug)]
pub struct Save {
    pub map_data: [u8; 0x317f],
    pub unknown: [u8; 0xa2],
    pub dialogue: [u8; 0x11f8],
    pub data_segment: DataSegment,
}

#[derive(BinRead, Debug)]
pub struct Sietch {
    pub first_name: u8,
    pub last_name: u8,
    pub desert: u8,
    pub map_x: u8,
    pub map_y: u8,
    pub map_u: u8,
    pub another_x: u8,
    pub another_y: u8,
    pub apparence: u8,
    pub troop_id: u8,
    pub status: u8,
    pub discoverable_at_phase: u8,
    pub unk1: u8,
    pub unk2: u8,
    pub unk3: u8,
    pub unk4: u8,
    pub spice_field_id: u8,
    pub unk5: u8,
    pub spice_density: u8,
    pub unk6: u8,
    pub nbr_moiss: u8,
    pub nbr_orni: u8,
    pub nbr_knife: u8,
    pub nbr_guns: u8,
    pub nbr_mods: u8,
    pub nbr_atoms: u8,
    pub nbr_bulbs: u8,
    pub water: u8,
}

#[derive(BinRead, Debug)]
pub struct DataSegment {
    pub rand_bits: u16,
    pub game_time: u16,
    pub current_location_and_room: u16,

    #[bin_read(offset = 0x0010)]
    pub persons_traveling_with: u16,
    pub persons_in_room: u16,
    pub persons_talking_to: u16,

    #[bin_read(offset = 0x002a)]
    pub game_phase: u8,

    #[bin_read(offset = 0x00cf)]
    pub days_left_until_spice_shipment: u8,

    #[bin_read(offset = 0x00e8)]
    pub ui_head_index: u8,

    #[bin_read(offset = 0x0100)]
    pub sietches: [Sietch; 70],

    #[bin_read(offset = 0x11dd)]
    pub intro_scene_28_attack_sprite_list: UISpriteList<2>,

    #[bin_read(offset = 0x120b)]
    pub palace_plan_sprite_list: UISpriteList<4>,
}

#[derive(BinRead, Debug)]
pub struct UISpriteList<const N: usize> {
    pub icons: [UISprite; N],
    pub end_marker: i16,
}

#[derive(BinRead, Debug)]
pub struct UISprite {
    pub index: u16,
    pub y: i16,
    pub x: i16,
}
