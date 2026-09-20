use rubiks_cube_lib::*;

const FMC: SteppedMethod<1, StepByPiece> = SteppedMethod([do_all]);

fn main() {
    let scrambled: NxN<3> = NxN::<3>::random_state();

    let scramble: String = FMC
        .solve(scrambled.inverse())
        .map(|(step, step_name)| {
            (step
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<String>>()
                .join(" ")
                + "\t\\")
                + &step_name
                + "\n"
        }) // Iter<String>
        .collect();

    let recon: String = Roux {
        cmll: one_look,
        ..Default::default()
    }
    .solve(scrambled)
    .to_recon();

    let cfop_recon: String = CFOP::default().solve(scrambled).to_recon();
}
