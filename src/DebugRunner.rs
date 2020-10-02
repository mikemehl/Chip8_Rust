use std::io::{stdin, Read};
use crate::ChipEight;

pub fn run(mut em : &mut ChipEight::Em)
{
  let mut cycles : u32 = 0;
  loop
  {
    cycles = cycles + 1;
    println!("////////////Cycle {}", cycles);
    cycle_once(&mut em);
    get_input();
  }
}

fn get_input()
{
  stdin().read(&mut [0]).unwrap();
}

fn cycle_once(em : &mut ChipEight::Em)
{
  let raw_op = em.cycle();
  em.print_debug_info(raw_op);
}
