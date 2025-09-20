use std::{env, fs, io::Cursor};

use bin_read::BinRead;
use savegame::{
    data::{Room, Save, Sietch},
    decompress_sav,
};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <savegame_file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    let input = fs::read(path)?;

    let unparsed_savegame = decompress_sav(&input)?;

    println!("Savegame file: {}", path);
    println!("Game time: {}", unparsed_savegame.gametime);
    println!(
        "Decompressed data size: {} bytes",
        unparsed_savegame.data.len()
    );

    let mut r = Cursor::new(&unparsed_savegame.data);
    let data = Save::bin_read(&mut r).unwrap();

    let ds = &data.data_segment;
    println!("rand_bits: {:04x}", ds.rand_bits);
    println!("game_time: {:04x}", ds.game_time);
    println!(
        "current_location_and_room: {:04x}",
        ds.current_location_and_room
    );

    for s in &ds.sietches {
        display_sietch(s);
    }

    println!("Palace rooms: (* = locked)");
    for (i, r) in ds.palace_rooms.iter().enumerate() {
        let name = match i + 1 {
            1 => "Outside",
            2 => "Equipment room",
            3 => "Greenhouse",
            4 => "Conference room",
            5 => "Balcony",
            6 => "Weapons room",
            7 => "Corridor",
            8 => "Communications room",
            9 => "Bed room",
            10 => "Throne room",
            11 => "Empty room",
            12 => "Communications room corridor",
            _ => "",
        };

        println!(
            "\troom {:2}: {:02x}, n={:4}{}, e={:4}{}, s={:4}{}, w={:4}{} {}",
            i + 1,
            r.room,
            r.north & 0x7f,
            if r.north & 0x80 == 0 { ' ' } else { '*' },
            r.east & 0x7f,
            if r.east & 0x80 == 0 { ' ' } else { '*' },
            r.south & 0x7f,
            if r.south & 0x80 == 0 { ' ' } else { '*' },
            r.west & 0x7f,
            if r.west & 0x80 == 0 { ' ' } else { '*' },
            name,
        );
    }

    Ok(())
}

fn display_sietch(s: &Sietch) {
    let first_names = [
        "Arrakeen", "Carthag", "Tuono", "Habbanya", "Oxtyn", "Tsympo", "Bledan", "Ergsun", "Haga",
        "Cielago", "Sihaya", "Celimyn",
    ];
    let last_names = [
        "(Atreides)",
        "(Harkonnen)",
        "Tabr",
        "Timin",
        "Tuek",
        "Harg",
        "Clam",
        "Tsymyn",
        "Siet",
        "Pyons",
        "Pyort",
    ];

    let name = format!(
        "{}{}{}",
        first_names
            .get((s.first_name - 1) as usize)
            .cloned()
            .unwrap_or_default(),
        if s.last_name < 3 { ' ' } else { '-' },
        last_names
            .get((s.last_name - 1) as usize)
            .cloned()
            .unwrap_or_default()
    );

    println!("{name}");
    println!("====================");
    println!("\t               desert: {}", s.desert);
    println!("\t                map_x: {}", s.map_x);
    println!("\t                map_y: {}", s.map_y);
    println!("\t                map_u: {}", s.map_u);
    println!("\t            another_x: {}", s.another_x);
    println!("\t            another_y: {}", s.another_y);
    println!("\t            apparence: {}", s.apparence);
    println!("\t             troop_id: {}", s.troop_id);
    println!("\t               status: {}", s.status);
    println!("\tdiscoverable_at_phase: {}", s.discoverable_at_phase);
    println!("\t                 unk1: {}", s.unk1);
    println!("\t                 unk2: {}", s.unk2);
    println!("\t                 unk3: {}", s.unk3);
    println!("\t                 unk4: {}", s.unk4);
    println!("\t       spice_field_id: {}", s.spice_field_id);
    println!("\t                 unk5: {}", s.unk5);
    println!("\t        spice_density: {}", s.spice_density);
    println!("\t                 unk6: {}", s.unk6);
    println!("\t            nbr_moiss: {}", s.nbr_moiss);
    println!("\t             nbr_orni: {}", s.nbr_orni);
    println!("\t            nbr_knife: {}", s.nbr_knife);
    println!("\t             nbr_guns: {}", s.nbr_guns);
    println!("\t             nbr_mods: {}", s.nbr_mods);
    println!("\t            nbr_atoms: {}", s.nbr_atoms);
    println!("\t            nbr_bulbs: {}", s.nbr_bulbs);
    println!("\t                water: {}", s.water);
    println!();
}
