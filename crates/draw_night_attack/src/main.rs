#![feature(random)]

use dune::attack::AttackState;

fn main() {
    let mut game_state = AttackState::default();

    game_state.set_rand_bits(std::random::random(..));
    game_state.set_rng_seed(std::random::random(..));
    game_state.set_masked_rng_seed(std::random::random(..));

    for frame_number in 0..660 {
        game_state.step_frame();

        let pal = game_state.pal();
        let fb = game_state.framebuffer();

        fb.write_ppm_scaled(pal, &format!("ppm/night_attack-{frame_number:05}.ppm"))
            .unwrap();
    }
}
