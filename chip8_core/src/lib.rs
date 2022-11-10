use std::{ops::{Div}};
use rand::random;

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

        // important gor fx29 instruction
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
                let x = d2 as usize;
                let kk = (op & 0xFF) as u8;
                if self.v_reg[x] != kk {
                    self.pc += 2;
                }
            },
            (5, _, _, 0) => {
                // skip next instruction if Vx = Vy
                // 5xy0
                let x = d2 as usize;
                let y = d3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },
            (6, _, _, _) => {
                // set Vx = kk
                // 6xkk
                let x = d2 as usize;
                let kk  = (op & 0xFF) as u8;
                self.v_reg[x] = kk;
            },
            (7, _, _, _) =>  {
                // set Vx = Vx + kk
                // 7xkk
                self.v_reg[d2 as usize] += (op & 0xFF) as u8;
            },
            (8, _, _, 0) => {
                // set Vx = Vy
                // 8xy0
                self.v_reg[d2 as usize] = self.v_reg[d3 as usize];
            },
            (8, _, _, 1) => {
                // set Vx = Vx or Vy
                // 8xy1
                self.v_reg[d2 as usize] |= self.v_reg[d3 as usize];
            },
            (8, _, _, 2) => {
                // set Vx = Vx and Vy
                // 8xy2
                self.v_reg[d2 as usize] &= self.v_reg[d3 as usize];
            },
            (8, _, _, 3) => {
                // set Vx = Vx xor Vy
                // 8xy3
                self.v_reg[d2 as usize] ^= self.v_reg[d3 as usize];
            },
            (8, _, _, 4) => {
                // sets Vx = Vx + Vy, set VF = carry
                // Values of Vx and Vy are added together.  If reult is greater than 8 bits, VF is set to 1, otherwise 0.  Lowest 8 bits are saved in Vx
                // 8xy4
                let x = self.v_reg[d2 as usize];
                let y = self.v_reg[d3 as usize];

                let (new_x, carry) =x.overflowing_add(y);
                self.v_reg[0xF] =  if carry {1} else {0};
                self.v_reg[d2 as usize] = new_x;
            },
            (8, _, _, 5) => {
                // Set Vx = Vx - Vy, set VF = NOT borrow
                // if Vx > Vy, then VF is set to 1, otherwise 0.  Then Vy is subtracted from Vx, result is stored in Vx
                // 8xy5
                let x = self.v_reg[d2 as usize];
                let y = self.v_reg[d3 as usize];
                self.v_reg[0xF] = if x > y {1} else {0};
                self.v_reg[d2 as usize] = (y - x) & 0xFF;
            },
            (8, _, _, 6) => {
                // Set Vx = Vx SHR1
                // if the least-signigicant bit of Vx is 1, then VF is set to 1, otherwise 0.  THen Vx is divided by 2
                // 8xy6
                let x = self.v_reg[d2 as usize];
                let y = self.v_reg[d3 as usize];

                self.v_reg[0xF] =  if (x & 0xF) == 1 {1} else {0};
                self.v_reg[d2 as usize] =  x.div(y);
            },
            (8, _, _, 7) => {
                // Set Vx = Vy - Vx, set Vx = NOT borrow
                // if Vy > Vx, then VF is set to 1 otherwise 0.  Results stored in Vx
                // 8xy7

                let x = self.v_reg[d2 as usize];
                let y = self.v_reg[d3 as usize]; 
                let (new_x, borrow) = y.overflowing_sub(x);

                self.v_reg[0xF] = if y > x {1} else {0};
                self.v_reg[d2 as usize] = new_x;
            }, 
            (8, _, _, 0xE) => {
                // Set Vx = Vx SHL 1.
                // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
                // 8xyE
                let x = self.v_reg[d2 as usize];
                let y = self.v_reg[d3 as usize]; 

                self.v_reg[0xF] = if (x & 0xF0) == 1 {1} else {0};
                self.v_reg[d2 as usize] = x * y;
            },
            (9, _, _, 0) => {
                // Skip next instruction if Vx != Vy.
                // The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2
                // 9xy0
                if self.v_reg[d2 as usize] == self.v_reg[d3 as usize] {
                    return
                }
                self.pc += 2;
            }, 
            (0xA, _, _, _) => {
                // Set I = nnn.
                // The value of register I is set to nnn.
                // Annn
                self.i_reg =  (op & 0xFFF) as u16;
            },
            (0xB, _, _, _) => {
                // Jump to location nnn + V0.
                // The program counter is set to nnn plus the value of V0.
                // Bnnn
                self.pc = (op & 0xFFF) + self.v_reg[0] as u16;
            },
            (0xC, _, _, _) => {
                // Set Vx = random byte AND kk.
                // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. 
                // The results are stored in Vx. See instruction 8xy2 for more information on AND.
                // Cxkk
                let kk = (op & 0xFF) as u8;
                let x = d2 as usize;
                let rng = rand::random::<u8>();
                self.v_reg[x] = rng & kk;
            },
            (0xD, _, _, _) => {
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                // The interpreter reads n bytes from memory, starting at the address stored in I. 
                // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). 
                // Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. 
                // If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. 
                // See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
                // Dxyn
                
                // x, y coord 
                let x_coord = self.v_reg[d2 as usize] as u16;
                let y_coord = self.v_reg[d3 as usize] as u16;

                // n = dteremins how many rows high our sprit is
                let rows = d4;
                // keep track if any pixels were flipped
                let mut flipped = false;
                //iter over each row of sprite
                for y_line in 0..rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0x80 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;

                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                self.v_reg[0xF] = if flipped {1} else {0};
            },
            (0xE, _, 9, 0xE) => {
                // Ex9E
                // Skip if keys pressed

                let key_idx = self.v_reg[d2 as usize];
                if self.keys[key_idx as usize] {
                    self.pc += 2
                } 
            },
            (0xE, _, 0xA, 1) => {
                //Skip if keys not pressed
                // ExA1
                let key_idx = self.v_reg[d2 as usize];
                if !self.keys[key_idx as usize] {
                    self.pc += 2
                } 
            },
            (0xF, _, 0, 7) => {
                // Fx07
                // set Vx to delay timer value
                self.v_reg[d2 as usize] = self.dt;
            },
            (0xF, _, 0, 0xA) => {
                // Fx0A
                // Wait for key press - blocks until a key is prssed
                // When more than one key prssed, lowest indexed is used.  This key is stored in Vx
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc += 2;
                }
            },
            (0xF, _, 1, 5) => {
                // Fx15 
                // Dt = Vx
                let new_val = self.v_reg[d2 as usize];
                self.dt = new_val;
            },
            (0xF, _, 1, 8) => {
                // Fx18
                // St = Vx
                let new_val = self.v_reg[d2 as usize];
                self.st = new_val;
            },
            (0xF, _, 1, 0xE) => {
                // Fx1E
                // I += Vx
                // if overflow, register should simply roll over to 0.  (rusts wrapping_add)
                self.v_reg[d2 as usize] = self.v_reg[d2 as usize].wrapping_add(self.i_reg as u8);
            }, 
            (0xF, _, 2, 9) => {
                // Fx29
                // Set I to Font Address
                // fonts are stored in the first sections of ram
                // we are multiplying by 5 since each font is 5 bytes long
                self.i_reg = self.v_reg[d2 as usize] as u16 * 5;
            },
            (0xF, _, 3, 3) => {
                // Fx33
                // i = BCD of Vx (BCD - binary coded decimal)
                
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
