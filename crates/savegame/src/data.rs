use bin_read::BinRead;

#[derive(BinRead, Debug)]
pub struct Save {
    pub map_data: [u8; 50684 / 4],
    pub unknown: [u8; 162],
    pub dialogue: [u8; 4600],
    pub data_segment: DataSegment,
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
