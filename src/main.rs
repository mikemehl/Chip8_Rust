mod ChipEight;
mod DebugRunner;

fn main() {
    let mut em = ChipEight::new_em();
    println!("Hello, world!");
    let fname = "chip8_roms/test_opcode.ch8";
    em.load_cart(fname.to_string());

    println!("Starting emulation...");
    DebugRunner::run(&mut em);
}
