use dune::attack::GameState;

fn main() {
    let mut game_state = GameState::default();

    for frame_number in 0..660 {
        game_state.step_frame();

        let pal = game_state.pal();
        let fb = game_state.framebuffer();

        fb.write_ppm_scaled(pal, &format!("ppm/night_attack-{frame_number:05}.ppm"))
            .unwrap();
    }
}
