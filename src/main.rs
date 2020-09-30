mod ChipEight;

fn main() {
    let mut em = ChipEight::new_em();
    println!("Hello, world!");
    let fname = "/mnt/c/Users/micha/Dropbox/Code/CHIP8_Rust/chip8_roms/test_opcode.ch8";
    em.load_cart(fname.to_string());
}
