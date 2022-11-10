const MEM_SIZE: usize = 0x1000;
const V_REG_SIZE: usize = 0x0F;
const STACK_SIZE: usize = 0x0F;
const KEYPAD_SIZE: usize = 0x0F;
const START_ADDR: u16 = 0x200; // start address for all chip 8 programs

const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct Chip8 {
    pc: u16,                                      // Program Counter
    ram: [u8; MEM_SIZE],                          // RAM
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // Display Screen
    v_reg: [u8; V_REG_SIZE],                      // V registers
    i_reg: u16,                                   // Indexing Register
    sp: u16,                                      // Stack pointer
    stack: [u16; STACK_SIZE],                     // CPU stack
    dt: u8,                                       // delay timer
    st: u8,                                       // sound timer
    keys: [bool; KEYPAD_SIZE],                    // Keypad
}

impl Chip8 {
    /// Chip 8 Initialization
    pub fn new() -> Self {
        let mut new_chip8 = Self {
            pc: START_ADDR,
            ram: [0; MEM_SIZE],
            screen: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
            v_reg: [0; V_REG_SIZE],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; KEYPAD_SIZE],
            dt: 0,
            st: 0,
        };

        new_chip8.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_chip8
    }

    /// Push u16 to stack
    pub fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    /// Pop u16 from stack
    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
        // possible underflow - panics
    }

    /// Reset chip8
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; MEM_SIZE];
        self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH],
        self.v_reg = [0; V_REG_SIZE];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; KEYPAD_SIZE];
        self.dt = 0;
        self.st = 0;
    }

    pub fn tick(&mut self) {
        // 1. Get value specified at memory address stored in Program Counter
        let op = self.fetch();
        // 2. Decode this instruction
        // 3. Execute
        self.execute(op);
        // 4. Move program counter to next instruction set
    }

    fn fetch(&mut self) -> u16 {
        // 4 bytes representing the instruction
        // most significant and least significant represnests the op code
        let high = self.ram[self.pc as usize] as u16;
        let low = self.ram[(self.pc + 1) as usize] as u16;
        let op = (high << 8) | low;

        // +2 since we are reading 4 bytes = 2 * u16 values
        self.pc += 2;
        
        op
    }

    fn execute(&mut self, op: u16) {
        let d1 = (op & 0xF000) >> 12;
        let d2 = (op & 0x0F00) >> 8;
        let d3 = (op & 0x00F0) >> 4;
        let d4 = op & 0x000F;

        match (d1, d2, d3, d4) {
            (0, 0, 0, 0) => return, // NOP
            (0, 0, 0xE, 0) => { self.screen = [false; SCREEN_HEIGHT * SCREEN_WIDTH]}  // clear screen
            (0, 0, 0xE, 0xE) => {
                // RET
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },
            (1, _, _, _) => {
                //JMP NNN
                let nnn = op & 0xFFF;
                self.pc = nnn;
            },
            (2, _, _, _) => {
                // CALL addr
                let addr = op & 0xFFF;
                self.push(self.pc);
                self.pc = addr;
            },
            (3, _, _ , _) => {
                // SKIP next if VX == NN
                // 3XNN

                let x = d2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2
                }
            },
            (4, _, _, _) => {
                // Skip next if Vx != kk
                // 4XKK
            }


            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op)
        }
    }

    pub fn tick_timers(&mut self ) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }
}

// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
