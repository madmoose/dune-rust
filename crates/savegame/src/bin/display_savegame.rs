use std::{env, fs, io::Cursor};

use bin_read::BinRead;
use savegame::{
    data::{Save, Sietch},
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

    for i in 0..70 {
        display_sietch(&ds.sietches[i]);
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
    println!("               desert: {}", s.desert);
    println!("                map_x: {}", s.map_x);
    println!("                map_y: {}", s.map_y);
    println!("                map_u: {}", s.map_u);
    println!("            another_x: {}", s.another_x);
    println!("            another_y: {}", s.another_y);
    println!("            apparence: {}", s.apparence);
    println!("             troop_id: {}", s.troop_id);
    println!("               status: {}", s.status);
    println!("discoverable_at_phase: {}", s.discoverable_at_phase);
    println!("                 unk1: {}", s.unk1);
    println!("                 unk2: {}", s.unk2);
    println!("                 unk3: {}", s.unk3);
    println!("                 unk4: {}", s.unk4);
    println!("       spice_field_id: {}", s.spice_field_id);
    println!("                 unk5: {}", s.unk5);
    println!("        spice_density: {}", s.spice_density);
    println!("                 unk6: {}", s.unk6);
    println!("            nbr_moiss: {}", s.nbr_moiss);
    println!("             nbr_orni: {}", s.nbr_orni);
    println!("            nbr_knife: {}", s.nbr_knife);
    println!("             nbr_guns: {}", s.nbr_guns);
    println!("             nbr_mods: {}", s.nbr_mods);
    println!("            nbr_atoms: {}", s.nbr_atoms);
    println!("            nbr_bulbs: {}", s.nbr_bulbs);
    println!("                water: {}", s.water);
    println!();
}
