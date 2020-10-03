use std::fs;
use std::io::{stdin, Read};
use std::thread;
use std::time;
use rand::prelude::*;

const MEM_LENGTH : usize = 0x1000;
const NUM_REGS : usize = 0x10;
const STACK_LEN : usize = 12;
const SCREEN_SIZE : usize = 64*32;
const PC_STEP : u16 = 2;
const PC_START : u16 = 0x200;
const CYCLE_HZ : f32 = 500e0;
const CYCLE_SEC : f32 = 1e0 / CYCLE_HZ;
const CYCLE_MSEC : f32 = CYCLE_SEC * 1e3;

pub struct Em 
{
  mem : [u8 ; MEM_LENGTH],
  v : [u8 ; NUM_REGS],
  i : u16,
  pc : u16,
  sp : u8,
  stack : [u16 ; STACK_LEN],
  delay_timer : u8,
  sound_timer : u8,
  screen_buf : [u8 ; SCREEN_SIZE],
  end_em : bool
}

struct EmOp
{
  nibbles  : [u16; 4],
  args_nnn : u16,
  args_nn  : u8,
  args_x   : u8,
  args_y   : u8,
  args_n   : u8
}

pub fn new_em() -> Em 
{
  Em
  {
    mem: [0x00; MEM_LENGTH],
    v: [0x00; NUM_REGS],
    i: 0,
    pc: PC_START,
    sp: 0,
    stack: [0x00; STACK_LEN],
    delay_timer: 0,
    sound_timer: 0,
    screen_buf: [0x00; SCREEN_SIZE],
    end_em: false,
  }
}

impl Em 
{

  pub fn load_cart(self : &mut Self, fname : String)
  {
    // Read the cart into memory.
    println!("Reading cart {}...", fname);
    let mut f = fs::File::open(fname).expect("Error opening file!");
    f.read(&mut self.mem[0x200..]).expect("Error reading file!"); // TODO: Make sure this is right...
    self.pc = PC_START;
  }

  pub fn cycle(self : &mut Self) -> u16
  {
    self.update_timers();
    let raw_op = self.fetch();
    let em_op = self.decode(&raw_op);
    self.execute(&em_op);
    //thread::sleep(time::Duration::from_millis(1000)); // TODO: Update this to run at 500hz, probably with more than just a sleep??
    raw_op
  }

  pub fn finished(self : &mut Self) -> bool
  {
    self.end_em
  }

  fn fetch(self : &mut Self) -> u16
  {
    let curr_opcode = (self.mem[self.pc as usize] as u16) << 8 | (self.mem[(self.pc + 1) as usize] as u16); 
    if self.pc as usize >= self.mem.len()
    {
       self.pc = PC_START;
    }
    curr_opcode
  }

  fn decode(self : &mut Self, op : &u16) -> EmOp
  {
    // Assign possible arguments and return..
    EmOp
    {
      nibbles  :  [(op & 0xF000) >> 12, (op & 0x0F00) >> 8, (op & 0x00F0) >> 4, (op & 0x000F)],
      args_nnn :   op & 0x0FFF,
      args_nn  :  (op & 0x00FF) as u8,
      args_x   : ((op & 0x0F00) >> 8) as u8,
      args_y   : ((op & 0x00F0) >> 4) as u8,
      args_n   :  (op & 0x000F) as u8
    }
    
  }

  fn execute(self : &mut Self, op : &EmOp)
  {
    match (op.nibbles[0], op.nibbles[1], op.nibbles[2], op.nibbles[3])
    {
       (0x0, 0x0, 0x0, 0x0) => self.end_em = true,
       (0x0, 0x0, 0xE, 0x0) => self.op_cls(),
       (0x0, 0x0, 0xE, 0xE) => self.op_ret(), 
       (0x0,   _,   _,   _) => self.pc = self.pc + PC_STEP,
       (0x1,   _,   _,   _) => self.op_jmp(op.args_nnn),
       (0x2,   _,   _,   _) => self.op_callsub(op.args_nnn), 
       (0x3,   _,   _,   _) => self.op_skipif(op.args_x, op.args_nn),
       (0x4,   _,   _,   _) => self.op_skipnif(op.args_x, op.args_nn),
       (0x5,   _,   _,   _) => self.op_skipifreg(op.args_x, op.args_y),
       (0x6,   _,   _,   _) => self.op_streg(op.args_x, op.args_nn),
       (0x7,   _,   _,   _) => self.op_addreg(op.args_x, op.args_nn),
       (0x8,   _,   _, 0x0) => self.op_movxy(op.args_x, op.args_y),
       (0x8,   _,   _, 0x1) => self.op_xory(op.args_x, op.args_y),
       (0x8,   _,   _, 0x2) => self.op_xandy(op.args_x, op.args_y),
       (0x8,   _,   _, 0x3) => self.op_xxory(op.args_x, op.args_y),
       (0x8,   _,   _, 0x4) => self.op_addxy(op.args_x, op.args_y),
       (0x8,   _,   _, 0x5) => self.op_subxy(op.args_x, op.args_y),
       (0x8,   _,   _, 0x6) => self.op_shrxy(op.args_x, op.args_y),
       (0x8,   _,   _, 0x7) => self.op_subyx(op.args_x, op.args_y),
       (0x8,   _,   _, 0xE) => self.op_shlxy(op.args_x, op.args_y),
       (0x9,   _,   _, 0x0) => self.op_skipifnreg(op.args_x, op.args_y),
       (0xA,   _,   _,   _) => self.op_addrtoi(op.args_nnn),
       (0xB,   _,   _,   _) => self.op_jmppls(op.args_nnn),
       (0xC,   _,   _,   _) => self.op_rnd(op.args_x, op.args_nn),
       _ => 
       {
          let mystery_op = (op.nibbles[0] as u16) << 12 | (op.nibbles[1] as u16) << 8 | (op.nibbles[2] as u16) << 4 | op.nibbles[3] as u16;
          println!("Unsupported opcode {:#x?} found!", mystery_op);
	  self.pc = self.pc + PC_STEP;
       }
    }
    
  }

  fn update_timers(self : &mut Self)
  {
    if self.delay_timer > 0
    {
      self.delay_timer -= 1;
    }

    if self.sound_timer > 0
    {
      self.sound_timer -= 1;
    }
  }

  pub fn print_debug_info(self : &mut Self, last_op : u16)
  {
    println!("=======DBG PROCESSOR STATE==========");
    println!("LAST OP: {:#x?}", last_op);
    println!("REGS: 0:{:#x?} 1:{:#x?} 2:{:#x?} 3:{:#x?} 4:{:#x?} 5:{:#x?} 6:{:#x?} 7:{:#x?} 8:{:#x?} 9:{:#x?} A:{:#x?} B:{:#x?} C:{:#x?} D:{:#x?} E:{:#x?}",
      self.v[0], self.v[1], self.v[2], self.v[3], self.v[4], self.v[5], self.v[6], self.v[7], 
      self.v[8], self.v[9], self.v[0xA], self.v[0xB], self.v[0xC], self.v[0xD], self.v[0xE]);
    println!("PC: {:#x?}  SP: {:#x?}  DLY: {:#x?}  SND: {:#x?}", self.pc, self.sp, self.delay_timer, self.sound_timer);
    if self.sp > 0
    {
      println!("STACK:");
      for i in 0..self.sp - 1
      {
        println!("{:#x?}", self.stack[i as usize])
      }
    }
  }

  // AND SO BEGAN THE LEGENDARY OPCODES 
  fn op_cls(self : &mut Self)
  {
    for i in 0..self.screen_buf.len()
    {
      self.screen_buf[i] = 0x00;
    }
  }

  fn op_ret(self : &mut Self)
  {
    assert!(self.sp > 0);
    self.sp = self.sp - 1;
    self.pc = self.stack[self.sp as usize] + 1;
  }

  fn op_jmp(self : &mut Self, addr : u16)
  {
    assert!(addr <= 0xFFF && addr > 0x200);
    self.pc = addr;
  }

  fn op_callsub(self : &mut Self, addr : u16)
  {
    assert!(addr <= 0xFFF && addr > 0x200);
    assert!((self.sp as usize) < self.stack.len());
    self.stack[self.sp as usize] = self.pc;
    self.sp = self.sp + 1;
    self.pc = addr;
  }

  fn op_skipif(self : &mut Self, reg : u8, val : u8)
  {
    let r = reg as usize;
    assert!(r < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    if self.v[r] == val
    {
      self.pc = self.pc + PC_STEP;
    }
  }

  fn op_skipnif(self : &mut Self, reg : u8, val : u8)
  {
    let r = reg as usize;
    assert!(r < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    if self.v[r] != val
    {
      self.pc = self.pc + PC_STEP;
    }
  }

  fn op_skipifreg(self : &mut Self, regx : u8, regy : u8)
  {
    let x = regx as usize;
    let y = regy as usize;
    assert!(x < NUM_REGS && y < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    if self.v[x] == self.v[y] 
    {
      self.pc = self.pc + PC_STEP;
    }
  }

  fn op_streg(self : &mut Self, reg : u8, val : u8)
  {
    let r = reg as usize;
    assert!(r < NUM_REGS);
    self.v[r] = val;
    self.pc = self.pc + PC_STEP;
  }

  fn op_addreg(self : &mut Self, reg : u8, val : u8)
  {
    let r = reg as usize;
    assert!(r < NUM_REGS);
    self.v[r] = self.v[r] + 1;
    self.pc = self.pc + PC_STEP;
  }

  fn op_movxy(self : &mut Self, regx : u8, regy : u8)
  {
    let x = regx as usize;
    let y = regy as usize;
    assert!(x < NUM_REGS && y < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    self.v[x] = self.v[y];
  }

  fn op_xory(self : &mut Self, regx : u8, regy : u8)
  {
    let x = regx as usize;
    let y = regy as usize;
    assert!(x < NUM_REGS && y < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    self.v[x] = self.v[y] | self.v[x];
  }

  fn op_xandy(self : &mut Self, regx : u8, regy : u8)
  {
    let x = regx as usize;
    let y = regy as usize;
    assert!(x < NUM_REGS && y < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    self.v[x] = self.v[y] & self.v[x];
  }

  fn op_xxory(self : &mut Self, regx : u8, regy : u8)
  {
    let x = regx as usize;
    let y = regy as usize;
    assert!(x < NUM_REGS && y < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    self.v[x] = self.v[y] ^ self.v[x];
  }

  fn op_addxy(self : &mut Self, regx : u8, regy : u8)
  {
    let x = regx as usize;
    let y = regy as usize;
    assert!(x < NUM_REGS && y < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    let val : u16 = (self.v[x] + self.v[y]).into(); 
    if val & 0xFF00 > 0
    {
       self.v[0xF] = 0x01;
    }
    else
    {
       self.v[0xF] = 0x00;
    }
    self.v[x] = (val & 0xFF00) as u8;
  }

  fn op_subxy(self : &mut Self, regx : u8, regy : u8)
  {
    let x = regx as usize;
    let y = regy as usize;
    assert!(x < NUM_REGS && y < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    let val : u16 = (self.v[x] - self.v[y]).into(); 
    if val & 0xFF00 > 0
    {
       self.v[0xF] = 0x00;
    }
    else
    {
       self.v[0xF] = 0x01;
    }
    self.v[x] = (val & 0xFF00) as u8;
  }

  fn op_shrxy(self : &mut Self, regx : u8, regy : u8)
  {
    let x = regx as usize;
    let y = regy as usize;
    assert!(x < NUM_REGS && y < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    self.v[0xF] = self.v[y] & 0x01;
    self.v[x] = self.v[y] >> 1;
  }

  fn op_subyx(self : &mut Self, regx : u8, regy : u8)
  {
    let x = regx as usize;
    let y = regy as usize;
    assert!(x < NUM_REGS && y < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    let val : u16 = (self.v[y] - self.v[x]).into(); 
    if val & 0xFF00 > 0
    {
       self.v[0xF] = 0x00;
    }
    else
    {
       self.v[0xF] = 0x01;
    }
    self.v[x] = (val & 0xFF00) as u8;
  }

  fn op_shlxy(self : &mut Self, regx : u8, regy : u8)
  {
    let x = regx as usize;
    let y = regy as usize;
    assert!(x < NUM_REGS && y < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    self.v[0xF] = self.v[y] & 0x80;
    self.v[x] = self.v[y] << 1;
  }

  fn op_skipifnreg(self : &mut Self, regx : u8, regy : u8)
  {
    let x = regx as usize;
    let y = regy as usize;
    assert!(x < NUM_REGS && y < NUM_REGS);
    self.pc = self.pc + PC_STEP;
    if self.v[x] != self.v[y] 
    {
      self.pc = self.pc + PC_STEP;
    }
  }

  fn op_addrtoi(self : &mut Self, addr : u16)
  {
    assert!((addr as usize) < MEM_LENGTH);
    self.i = addr;
    self.pc = self.pc + PC_STEP;
  
  }

  fn op_jmppls(self : &mut Self, addr : u16)
  {
    assert!((addr as usize) < MEM_LENGTH);
    self.pc = addr + (self.v[0] as u16);
  }

  fn op_rnd(self : &mut Self, reg : u8, mask : u8)
  {
    let mut rng = rand::thread_rng();
    self.v[reg as usize] = rng.gen::<u8>() & mask;
    self.pc = self.pc + PC_STEP;
  } 
}
