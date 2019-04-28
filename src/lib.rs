const RAM_SIZE: usize = 0x1000;
const STACK_SIZE: usize = 16;
const DISPLAY_HEIGHT: usize = 32;
const NUM_REGISTERS: usize = 16;
const MEM_PROGRAM_START: u16 = 0x200;

enum Instruction {
    SYS(u16),
    CLS,
    RET,
    JP(u16),
    CALL(u16),
    SE(u8, u8),

    DRW(u8, u8, u8),
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
        }
    }

    pub fn execute(&mut self, instr: Instruction) {
        match instr {
            Instruction::CLS => {
                for row in self.display.iter_mut() {
                    *row = 0;
                }
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
            }
            _ => {}
        }
    }
}

fn create_memory() -> [u8; RAM_SIZE] {
    let array = [0; RAM_SIZE];
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
    vm.execute(Instruction::DRW(0, 0, 0));
    println!("We did it");
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn instr_sys() {
        let mut vm = VM::new();
        let instr = Instruction::SYS(1);
        vm.execute(instr);

        // this instruction should be ignored
        assert_eq!(vm.reg_pc, 0);
    }

    #[test]
    fn instr_cls() {
        let mut vm = VM::new();

        vm.display[0] = 0b1111;

        let instr = Instruction::CLS;
        vm.execute(instr);

        // should clear display
        assert_eq!(vm.display[0], 0);
    }

    #[test]
    fn instr_drw() {
        let mut vm = VM::new();

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
        assert_eq!(vm.gen_registers[0xF], 0)
    }

    #[test]
    fn instr_drw_collision() {
        let mut vm = VM::new();

        vm.display[0] = 0b1;
        vm.memory[MEM_PROGRAM_START as usize] = 0b1;
        vm.reg_i = MEM_PROGRAM_START;
        vm.gen_registers[0 as usize] = 56;
        vm.gen_registers[1 as usize] = 0;

        vm.execute(Instruction::DRW(0, 1, 1));

        assert_eq!(vm.display[0], 0);
        assert_eq!(vm.gen_registers[0xF], 1)
    }
}
