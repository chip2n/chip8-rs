#![feature(wait_until)]

mod keys;
mod render;

use rand::rngs::mock::StepRng;
use rand::Rng;
use std::thread;
use std::time::Duration;

const RAM_SIZE: usize = 0x1000;
const STACK_SIZE: usize = 16;
const DISPLAY_HEIGHT: usize = 32;
const NUM_REGISTERS: usize = 16;
const MEM_PROGRAM_START: u16 = 0x200;

pub struct Program {
    pub instructions: Vec<Instruction>,
}

pub enum Instruction {
    SYS(u16),
    CLS,
    RET,
    JP(u16),
    CALL(u16),
    SE(u8, u8),
    SNE(u8, u8),
    SE2(u8, u8),
    LD(u8, u8),
    ADD(u8, u8),
    LD2(u8, u8),
    OR(u8, u8),
    AND(u8, u8),
    XOR(u8, u8),
    ADD2(u8, u8),
    SUB(u8, u8),
    SHR(u8, u8),
    SUBN(u8, u8),
    SHL(u8, u8),
    SNE2(u8, u8),
    LDI(u16),
    JPV0(u16),
    RND(u8, u8),
    DRW(u8, u8, u8),
    SKP(u8),
    SKNP(u8),
    LD3(u8),
    LD4(u8),
    LD5(u8),
    LD6(u8),
    ADD3(u8),
    LD7(u8),
}

struct VM {
    memory: [u8; RAM_SIZE],
    stack: [u16; STACK_SIZE],
    display: [u64; 32],
    gen_registers: [u8; NUM_REGISTERS],
    reg_i: u16,
    reg_pc: u16,
    reg_sp: u8,
    reg_delay: u8,
    reg_sound: u8,
    rng: Box<dyn rand::RngCore>,

    keyboard: keys::Keyboard,
}

impl VM {
    pub fn new() -> VM {
        let memory = create_memory();
        let stack = create_stack();
        let display = create_display();
        let gen_registers = create_gen_registers();

        VM {
            memory,
            stack,
            display,
            gen_registers,
            reg_i: 0,
            reg_pc: 0,
            reg_sp: 0,
            reg_delay: 0,
            reg_sound: 0,
            rng: Box::new(rand::thread_rng()),

            keyboard: keys::Keyboard::new(),
        }
    }

    //pub fn run(&mut self, program: Program) {}

    pub fn execute(&mut self, instr: Instruction) {
        match instr {
            Instruction::CLS => {
                for row in self.display.iter_mut() {
                    *row = 0;
                }
                self.reg_pc += 1;
            }
            Instruction::RET => {
                self.reg_pc = self.stack[self.reg_sp as usize];
                self.reg_sp -= 1;
            }
            Instruction::JP(addr) => {
                self.reg_pc = addr;
            }
            Instruction::CALL(addr) => {
                self.reg_sp += 1;
                self.stack[self.reg_sp as usize] = self.reg_pc;
                self.reg_pc = addr;
            }
            Instruction::SE(x, byte) => {
                if self.gen_registers[x as usize] == byte {
                    self.reg_pc += 2;
                } else {
                    self.reg_pc += 1;
                }
            }
            Instruction::SNE(x, byte) => {
                if self.gen_registers[x as usize] != byte {
                    self.reg_pc += 2;
                } else {
                    self.reg_pc += 1;
                }
            }
            Instruction::SE2(x, y) => {
                if self.gen_registers[x as usize] == self.gen_registers[y as usize] {
                    self.reg_pc += 2;
                } else {
                    self.reg_pc += 1;
                }
            }
            Instruction::LD(x, byte) => {
                self.gen_registers[x as usize] = byte;
                self.reg_pc += 1;
            }
            Instruction::ADD(x, byte) => {
                self.gen_registers[x as usize] += byte;
                self.reg_pc += 1;
            }
            Instruction::LD2(x, y) => {
                self.gen_registers[x as usize] = self.gen_registers[y as usize];
                self.reg_pc += 1;
            }
            Instruction::OR(x, y) => {
                self.gen_registers[x as usize] =
                    self.gen_registers[x as usize] | self.gen_registers[y as usize];
                self.reg_pc += 1;
            }
            Instruction::AND(x, y) => {
                self.gen_registers[x as usize] =
                    self.gen_registers[x as usize] & self.gen_registers[y as usize];
                self.reg_pc += 1;
            }
            Instruction::XOR(x, y) => {
                self.gen_registers[x as usize] =
                    self.gen_registers[x as usize] ^ self.gen_registers[y as usize];
                self.reg_pc += 1;
            }
            Instruction::ADD2(x, y) => {
                let result =
                    self.gen_registers[x as usize] as u16 + self.gen_registers[y as usize] as u16;
                if result > 255 {
                    self.gen_registers[0xF] = 1;
                    self.gen_registers[x as usize] = result as u8;
                } else {
                    self.gen_registers[x as usize] = result as u8;
                    self.gen_registers[0xF] = 0;
                }
                self.reg_pc += 1;
            }
            Instruction::SUB(x, y) => {
                // TODO: Not sure this is the proper way to do subtraction
                if self.gen_registers[x as usize] > self.gen_registers[y as usize] {
                    let result = self.gen_registers[x as usize] - self.gen_registers[y as usize];
                    self.gen_registers[x as usize] = result;
                    self.gen_registers[0xF] = 1;
                } else {
                    let result = self.gen_registers[y as usize] - self.gen_registers[x as usize];
                    self.gen_registers[x as usize] = result;
                    self.gen_registers[0xF] = 0;
                }
                self.reg_pc += 1;
            }
            Instruction::SHR(x, _) => {
                if self.gen_registers[x as usize] % 2 == 0 {
                    self.gen_registers[0xF] = 0;
                } else {
                    self.gen_registers[0xF] = 1;
                }
                self.gen_registers[x as usize] = self.gen_registers[x as usize] >> 1;
                self.reg_pc += 1;
            }
            Instruction::SUBN(x, y) => {
                // TODO: Not sure this is the proper way to do subtraction
                if self.gen_registers[y as usize] > self.gen_registers[x as usize] {
                    let result = self.gen_registers[y as usize] - self.gen_registers[x as usize];
                    self.gen_registers[x as usize] = result;
                    self.gen_registers[0xF] = 1;
                } else {
                    let result = self.gen_registers[x as usize] - self.gen_registers[y as usize];
                    self.gen_registers[x as usize] = result;
                    self.gen_registers[0xF] = 0;
                }
                self.reg_pc += 1;
            }
            Instruction::SHL(x, _) => {
                if self.gen_registers[x as usize] >= 0b10000000 {
                    self.gen_registers[0xF] = 1;
                } else {
                    self.gen_registers[0xF] = 0;
                }
                self.gen_registers[x as usize] = self.gen_registers[x as usize] << 1;
                self.reg_pc += 1;
            }
            Instruction::SNE2(x, y) => {
                if self.gen_registers[x as usize] != self.gen_registers[y as usize] {
                    self.reg_pc += 2;
                } else {
                    self.reg_pc += 1;
                }
            }
            Instruction::LDI(addr) => {
                self.reg_i = addr;
                self.reg_pc += 1;
            }
            Instruction::JPV0(addr) => {
                self.reg_pc = addr + self.gen_registers[0] as u16;
            }
            Instruction::RND(x, byte) => {
                let value = (*(self.rng)).next_u32() as u8;
                self.gen_registers[x as usize] = value & byte;
                self.reg_pc += 1;
            }
            Instruction::DRW(x, y, n) => {
                let vx = self.gen_registers[x as usize];
                let vy = self.gen_registers[y as usize];
                let start = self.reg_i as usize;

                for i in 0..n {
                    let data = self.memory[start + i as usize];
                    let sprite_row = create_sprite_mask(data, vx);
                    let result = self.display[x as usize] ^ sprite_row;

                    if sprite_row & result == sprite_row {
                        self.gen_registers[0xF] = 0;
                    } else {
                        self.gen_registers[0xF] = 1;
                    }

                    self.display[(vy + i) as usize] = result;
                }
                self.reg_pc += 1;
            }
            Instruction::SKP(x) => {
                let key_num = self.gen_registers[x as usize];
                let key = keys::Key::from_num(key_num).unwrap();
                if self.keyboard.is_pressed(&key) {
                    self.reg_pc += 2;
                } else {
                    self.reg_pc += 1;
                }
            }
            Instruction::SKNP(x) => {
                let key_num = self.gen_registers[x as usize];
                let key = keys::Key::from_num(key_num).unwrap();
                if self.keyboard.is_pressed(&key) {
                    self.reg_pc += 1;
                } else {
                    self.reg_pc += 2;
                }
            }
            Instruction::LD3(x) => {
                self.gen_registers[x as usize] = self.reg_delay;
                self.reg_pc += 1;
            }
            Instruction::LD4(x) => {
                let key = self.keyboard.wait();
                self.gen_registers[x as usize] = key.to_num();
                self.reg_pc += 1;
            }
            Instruction::LD5(x) => {
                self.reg_delay = self.gen_registers[x as usize];
                self.reg_pc += 1;
            }
            Instruction::LD6(x) => {
                self.reg_sound = self.gen_registers[x as usize];
                self.reg_pc += 1;
            }
            Instruction::ADD3(x) => {
                self.reg_i = self.reg_i + self.gen_registers[x as usize] as u16;
                self.reg_pc += 1;
            }
            Instruction::LD7(x) => {
                let d = self.gen_registers[x as usize];
                self.reg_i = digit(d) as u16;
                self.reg_pc += 1;
            }
            _ => {}
        }
    }
}

fn digit(digit: u8) -> usize {
    (digit * 5) as usize
}

fn create_memory() -> [u8; RAM_SIZE] {
    let mut array = [0; RAM_SIZE];

    array[digit(0) + 0] = 0xF0;
    array[digit(0) + 1] = 0x90;
    array[digit(0) + 2] = 0x90;
    array[digit(0) + 3] = 0x90;
    array[digit(0) + 4] = 0xF0;

    array[digit(1) + 0] = 0x20;
    array[digit(1) + 1] = 0x60;
    array[digit(1) + 2] = 0x20;
    array[digit(1) + 3] = 0x20;
    array[digit(1) + 4] = 0x70;

    array[digit(2) + 0] = 0xF0;
    array[digit(2) + 1] = 0x10;
    array[digit(2) + 2] = 0xF0;
    array[digit(2) + 3] = 0x80;
    array[digit(2) + 4] = 0xF0;

    array[digit(3) + 0] = 0xF0;
    array[digit(3) + 1] = 0x10;
    array[digit(3) + 2] = 0xF0;
    array[digit(3) + 3] = 0x10;
    array[digit(3) + 4] = 0xF0;

    array[digit(4) + 0] = 0x90;
    array[digit(4) + 1] = 0x90;
    array[digit(4) + 2] = 0xF0;
    array[digit(4) + 3] = 0x10;
    array[digit(4) + 4] = 0x10;

    array[digit(5) + 0] = 0xF0;
    array[digit(5) + 1] = 0x80;
    array[digit(5) + 2] = 0xF0;
    array[digit(5) + 3] = 0x10;
    array[digit(5) + 4] = 0xF0;

    array[digit(6) + 0] = 0xF0;
    array[digit(6) + 1] = 0x80;
    array[digit(6) + 2] = 0xF0;
    array[digit(6) + 3] = 0x90;
    array[digit(6) + 4] = 0xF0;

    array[digit(7) + 0] = 0xF0;
    array[digit(7) + 1] = 0x10;
    array[digit(7) + 2] = 0x20;
    array[digit(7) + 3] = 0x40;
    array[digit(7) + 4] = 0x40;

    array[digit(8) + 0] = 0xF0;
    array[digit(8) + 1] = 0x90;
    array[digit(8) + 2] = 0xF0;
    array[digit(8) + 3] = 0x90;
    array[digit(8) + 4] = 0xF0;

    array[digit(9) + 0] = 0xF0;
    array[digit(9) + 1] = 0x90;
    array[digit(9) + 2] = 0xF0;
    array[digit(9) + 3] = 0x10;
    array[digit(9) + 4] = 0xF0;

    array[digit(10) + 0] = 0xF0;
    array[digit(10) + 1] = 0x90;
    array[digit(10) + 2] = 0xF0;
    array[digit(10) + 3] = 0x90;
    array[digit(10) + 4] = 0x90;

    array[digit(11) + 0] = 0xE0;
    array[digit(11) + 1] = 0x90;
    array[digit(11) + 2] = 0xE0;
    array[digit(11) + 3] = 0x90;
    array[digit(11) + 4] = 0xE0;

    array[digit(12) + 0] = 0xF0;
    array[digit(12) + 1] = 0x80;
    array[digit(12) + 2] = 0x80;
    array[digit(12) + 3] = 0x80;
    array[digit(12) + 4] = 0xF0;

    array[digit(13) + 0] = 0xE0;
    array[digit(13) + 1] = 0x90;
    array[digit(13) + 2] = 0x90;
    array[digit(13) + 3] = 0x90;
    array[digit(13) + 4] = 0xE0;

    array[digit(14) + 0] = 0xF0;
    array[digit(14) + 1] = 0x80;
    array[digit(14) + 2] = 0xF0;
    array[digit(14) + 3] = 0x80;
    array[digit(14) + 4] = 0xF0;

    array[digit(15) + 0] = 0xF0;
    array[digit(15) + 1] = 0x80;
    array[digit(15) + 2] = 0xF0;
    array[digit(15) + 3] = 0x80;
    array[digit(15) + 4] = 0x80;

    array
}

fn create_stack() -> [u16; STACK_SIZE] {
    let array = [0; STACK_SIZE];
    array
}

fn create_display() -> [u64; DISPLAY_HEIGHT] {
    let array = [0; DISPLAY_HEIGHT];
    array
}

fn create_gen_registers() -> [u8; NUM_REGISTERS] {
    let array = [0; NUM_REGISTERS];
    array
}

fn create_sprite_mask(sprite: u8, x: u8) -> u64 {
    (sprite as u64) << (64 - 8 - x)
}

pub fn run() {
    let mut vm = VM::new();
    let renderer = render::Renderer::new(vm.keyboard.clone());

    let mut x = 0;

    vm.execute(Instruction::LD4(1));

    loop {
        vm.reg_i = 0x200;
        vm.gen_registers[0] = 10 + (x % 32);
        vm.gen_registers[1] = 20;
        vm.memory[0x200] = 5;
        vm.execute(Instruction::DRW(0, 1, 2));

        renderer.render(vm.display);
        thread::sleep(Duration::from_millis(100));

        x += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_vm() -> VM {
        VM::new()
    }

    #[test]
    fn execute_initial_pc() {
        let vm = create_vm();
        assert_eq!(vm.reg_pc, 0);
    }

    #[test]
    fn instr_sys() {
        let mut vm = create_vm();
        let instr = Instruction::SYS(1);
        vm.execute(instr);

        // this instruction should be ignored
        assert_eq!(vm.reg_pc, 0);
    }

    #[test]
    fn instr_cls() {
        let mut vm = create_vm();

        vm.display[0] = 0b1111;

        let instr = Instruction::CLS;
        vm.execute(instr);

        // should clear display
        assert_eq!(vm.display[0], 0);

        // should inc PC
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_ret() {
        let mut vm = create_vm();

        let stack_pc = 10;
        let sp = 5;
        vm.reg_sp = sp;
        vm.stack[5] = stack_pc;

        let instr = Instruction::RET;
        vm.execute(instr);

        assert_eq!(vm.reg_pc, stack_pc);
        assert_eq!(vm.reg_sp, sp - 1);
    }

    #[test]
    fn instr_jp() {
        let mut vm = create_vm();
        let addr = 10;
        let instr = Instruction::JP(addr);
        vm.execute(instr);

        assert_eq!(vm.reg_pc, addr);
    }

    #[test]
    fn instr_call() {
        let mut vm = create_vm();
        vm.reg_pc = 5;

        let addr = 10;
        let instr = Instruction::CALL(addr);
        vm.execute(instr);

        assert_eq!(vm.reg_sp, 1);
        assert_eq!(vm.stack[1], 5);
        assert_eq!(vm.reg_pc, addr);
    }

    #[test]
    fn instr_se_skip() {
        let mut vm = create_vm();
        vm.gen_registers[2] = 10;

        vm.execute(Instruction::SE(2, 10));

        assert_eq!(vm.reg_pc, 2);
    }

    #[test]
    fn instr_se_noskip() {
        let mut vm = create_vm();
        vm.gen_registers[2] = 9;

        vm.execute(Instruction::SE(2, 10));

        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_sne_skip() {
        let mut vm = create_vm();
        vm.gen_registers[2] = 9;

        vm.execute(Instruction::SNE(2, 10));

        assert_eq!(vm.reg_pc, 2);
    }

    #[test]
    fn instr_sne_noskip() {
        let mut vm = create_vm();
        vm.gen_registers[2] = 10;

        vm.execute(Instruction::SNE(2, 10));

        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_se2_skip() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 10;
        vm.gen_registers[2] = 10;

        vm.execute(Instruction::SE2(1, 2));

        assert_eq!(vm.reg_pc, 2);
    }

    #[test]
    fn instr_se2_noskip() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 9;
        vm.gen_registers[2] = 10;

        vm.execute(Instruction::SE2(1, 2));

        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_ld() {
        let mut vm = create_vm();
        vm.execute(Instruction::LD(3, 10));

        assert_eq!(vm.gen_registers[3], 10);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_add() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 1;

        vm.execute(Instruction::ADD(1, 10));

        assert_eq!(vm.gen_registers[1], 11);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_ld2() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 9;
        vm.gen_registers[2] = 10;
        vm.execute(Instruction::LD2(1, 2));

        assert_eq!(vm.gen_registers[1], 10);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_or() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 0b001;
        vm.gen_registers[2] = 0b011;
        vm.execute(Instruction::OR(1, 2));

        assert_eq!(vm.gen_registers[1], 0b011);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_and() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 0b001;
        vm.gen_registers[2] = 0b011;
        vm.execute(Instruction::AND(1, 2));

        assert_eq!(vm.gen_registers[1], 0b001);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_xor() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 0b001;
        vm.gen_registers[2] = 0b011;
        vm.execute(Instruction::XOR(1, 2));

        assert_eq!(vm.gen_registers[1], 0b010);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_add2_nooverflow() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 0b10000000;
        vm.gen_registers[2] = 0b01111111;
        vm.gen_registers[0xF] = 2; // to make sure register is set to zero
        vm.execute(Instruction::ADD2(1, 2));

        assert_eq!(vm.gen_registers[1], 0b11111111);
        assert_eq!(vm.gen_registers[0xF], 0);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_add2_overflow() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 0b10000000;
        vm.gen_registers[2] = 0b10000001;
        vm.execute(Instruction::ADD2(1, 2));

        assert_eq!(vm.gen_registers[1], 0b00000001);
        assert_eq!(vm.gen_registers[0xF], 1);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_sub_noborrow() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 3;
        vm.gen_registers[2] = 2;
        vm.execute(Instruction::SUB(1, 2));

        assert_eq!(vm.gen_registers[1], 1);
        assert_eq!(vm.gen_registers[0xF], 1);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_sub_borrow() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 2;
        vm.gen_registers[2] = 3;
        vm.gen_registers[0xF] = 2; // to make sure register is set to zero
        vm.execute(Instruction::SUB(1, 2));

        assert_eq!(vm.gen_registers[1], 1);
        assert_eq!(vm.gen_registers[0xF], 0);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_sub_equal() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 3;
        vm.gen_registers[2] = 3;
        vm.execute(Instruction::SUB(1, 2));

        assert_eq!(vm.gen_registers[1], 0);
        assert_eq!(vm.gen_registers[0xF], 0);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_shr_odd() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 0b111;
        vm.execute(Instruction::SHR(1, 2));

        assert_eq!(vm.gen_registers[1], 0b11);
        assert_eq!(vm.gen_registers[0xF], 1);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_shr_even() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 0b100;
        vm.gen_registers[0xF] = 2; // to make sure register is set to zero
        vm.execute(Instruction::SHR(1, 2));

        assert_eq!(vm.gen_registers[1], 0b10);
        assert_eq!(vm.gen_registers[0xF], 0);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_subn_noborrow() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 2;
        vm.gen_registers[2] = 3;
        vm.execute(Instruction::SUBN(1, 2));

        assert_eq!(vm.gen_registers[1], 1);
        assert_eq!(vm.gen_registers[0xF], 1);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_subn_borrow() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 3;
        vm.gen_registers[2] = 2;
        vm.gen_registers[0xF] = 2; // to make sure register is set to zero
        vm.execute(Instruction::SUBN(1, 2));

        assert_eq!(vm.gen_registers[1], 1);
        assert_eq!(vm.gen_registers[0xF], 0);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_subn_even() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 3;
        vm.gen_registers[2] = 3;
        vm.gen_registers[0xF] = 2; // to make sure register is set to zero
        vm.execute(Instruction::SUBN(1, 2));

        assert_eq!(vm.gen_registers[1], 0);
        assert_eq!(vm.gen_registers[0xF], 0);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_shr_nooverflow() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 0b11000000;
        vm.execute(Instruction::SHL(1, 2));

        assert_eq!(vm.gen_registers[1], 0b10000000);
        assert_eq!(vm.gen_registers[0xF], 1);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_shr_overflow() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 0b01000000;
        vm.gen_registers[0xF] = 2; // to make sure register is set to zero
        vm.execute(Instruction::SHL(1, 2));

        assert_eq!(vm.gen_registers[1], 0b10000000);
        assert_eq!(vm.gen_registers[0xF], 0);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_sne2_skip() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 8;
        vm.gen_registers[2] = 9;

        vm.execute(Instruction::SNE2(1, 2));

        assert_eq!(vm.reg_pc, 2);
    }

    #[test]
    fn instr_sne2_noskip() {
        let mut vm = create_vm();
        vm.gen_registers[1] = 10;
        vm.gen_registers[2] = 10;

        vm.execute(Instruction::SNE2(1, 2));

        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_ldi() {
        let mut vm = create_vm();
        vm.execute(Instruction::LDI(0x555));

        assert_eq!(vm.reg_i, 0x555);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_jpv0() {
        let mut vm = create_vm();
        vm.gen_registers[0] = 3;
        vm.execute(Instruction::JPV0(0x300));

        assert_eq!(vm.reg_pc, 0x303);
    }

    #[test]
    fn instr_rnd() {
        let mut vm = create_vm();
        vm.rng = Box::new(StepRng::new(0b110, 1));

        vm.execute(Instruction::RND(1, 0b101));

        assert_eq!(vm.gen_registers[1], 0b100);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_drw() {
        let mut vm = create_vm();

        let sprite1 = 0b11 << 3;
        let sprite2 = 0b11 << 2;
        let x: u8 = 1;
        let y: u8 = 2;
        let n: u8 = 2;
        let vx = 4;
        let vy = 5;

        vm.memory[MEM_PROGRAM_START as usize] = sprite1;
        vm.memory[MEM_PROGRAM_START as usize + 1] = sprite2;
        vm.reg_i = MEM_PROGRAM_START;
        vm.gen_registers[x as usize] = vx;
        vm.gen_registers[y as usize] = vy;

        vm.execute(Instruction::DRW(x, y, n));

        let expected1 = create_sprite_mask(sprite1, vx);
        let expected2 = create_sprite_mask(sprite2, vx);

        assert_eq!(vm.display[vy as usize], expected1);
        assert_eq!(vm.display[(vy + 1) as usize], expected2);
        assert_eq!(vm.gen_registers[0xF], 0);

        // should inc PC
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_drw_collision() {
        let mut vm = create_vm();

        vm.display[0] = 0b1;
        vm.memory[MEM_PROGRAM_START as usize] = 0b1;
        vm.reg_i = MEM_PROGRAM_START;
        vm.gen_registers[0 as usize] = 56;
        vm.gen_registers[1 as usize] = 0;

        vm.execute(Instruction::DRW(0, 1, 1));

        assert_eq!(vm.display[0], 0);
        assert_eq!(vm.gen_registers[0xF], 1);
    }

    #[test]
    fn instr_skp_pressed() {
        let mut vm = create_vm();
        vm.keyboard = keys::Keyboard::new();
        vm.keyboard.set_pressed(keys::Key::Key3);
        vm.gen_registers[1] = 3;

        vm.execute(Instruction::SKP(1));

        assert_eq!(vm.reg_pc, 2);
    }

    #[test]
    fn instr_skp_notpressed() {
        let mut vm = create_vm();
        vm.keyboard = keys::Keyboard::new();
        vm.keyboard.set_pressed(keys::Key::Key2);
        vm.gen_registers[1] = 3;

        vm.execute(Instruction::SKP(1));

        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_sknp_pressed() {
        let mut vm = create_vm();
        vm.keyboard = keys::Keyboard::new();
        vm.keyboard.set_pressed(keys::Key::Key3);
        vm.gen_registers[1] = 3;

        vm.execute(Instruction::SKNP(1));
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_sknp_notpressed() {
        let mut vm = create_vm();
        vm.keyboard = keys::Keyboard::new();
        vm.keyboard.set_pressed(keys::Key::Key2);
        vm.gen_registers[1] = 3;

        vm.execute(Instruction::SKNP(1));

        assert_eq!(vm.reg_pc, 2);
    }

    #[test]
    fn instr_ld3() {
        let mut vm = create_vm();
        vm.reg_delay = 3;
        vm.execute(Instruction::LD3(1));
        assert_eq!(vm.gen_registers[1], 3);
        assert_eq!(vm.reg_pc, 1);
    }

    // TODO: This test will probably fail occasionally - can we do better?
    #[test]
    fn instr_ld4() {
        let mut vm = create_vm();

        let keyboard2 = vm.keyboard.clone();
        thread::spawn(move || {
            keyboard2.set_pressed(keys::Key::Key4);
        });

        vm.execute(Instruction::LD4(1));

        assert_eq!(vm.reg_pc, 1);
        assert_eq!(vm.gen_registers[1], 4);
    }

    #[test]
    fn instr_ld5() {
        let mut vm = create_vm();
        vm.reg_delay = 3;
        vm.gen_registers[1] = 4;
        vm.execute(Instruction::LD5(1));
        assert_eq!(vm.reg_delay, 4);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_ld6() {
        let mut vm = create_vm();
        vm.reg_sound = 3;
        vm.gen_registers[1] = 4;
        vm.execute(Instruction::LD6(1));
        assert_eq!(vm.reg_sound, 4);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_add3() {
        let mut vm = create_vm();
        vm.reg_i = 3;
        vm.gen_registers[1] = 4;
        vm.execute(Instruction::ADD3(1));
        assert_eq!(vm.reg_i, 7);
        assert_eq!(vm.reg_pc, 1);
    }

    #[test]
    fn instr_ld7() {
        let mut vm = create_vm();
        vm.reg_i = 3;
        vm.gen_registers[1] = 4;
        vm.execute(Instruction::LD7(1));
        assert_eq!(vm.reg_i, digit(4) as u16);
        assert_eq!(vm.reg_pc, 1);
    }
}
