use std::{env, fs, io::Cursor};

use bin_read::BinRead;
use savegame::{data::Save, decompress_sav};

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

    println!("{:#?}", &data.data_segment);

    println!("gametime: {:04x}", data.data_segment.game_time);

    Ok(())
}
