use dune::{GameState, dat_file::DatFile};

fn main() {
    let Some(dat_path) = std::env::args().nth(1) else {
        print!("Specify dat file");
        return;
    };

    let Ok(dat_file) = DatFile::open(&dat_path) else {
        println!("Failed to open dat file '{dat_path}'");
        return;
    };

    let mut game = GameState { dat_file };

    game.play_intro_1();
}
