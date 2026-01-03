//! A tiny “toy CPU” emulator that stores registers in **big-endian** form.
//!
//! This demonstrates an important pattern when working with `simple_endian`:
//! - registers/memory store values in a declared endian form (here: BE)
//! - the emulator performs computations in native integers
//! - we convert using `to_native()` / `from_native()` at the boundaries
//!
//! Run:
//!   cargo run --example cpu_emulator --features "derive io-std"

#![cfg_attr(not(feature = "io-std"), allow(dead_code, unused_imports))]
#![cfg_attr(not(feature = "io-std"), allow(dead_code, unused_imports))]

#[cfg(feature = "io-std")]
mod real {
    use simple_endian::BigEndian;
    use simple_endian::u16be;
    use simple_endian::{read_specific, write_specific};
    use std::io::Cursor;

    /// Minimal CPU state.
    ///
    /// Registers are stored as *BigEndian tagged values* to model a BE CPU.
    #[derive(Debug, Clone)]
    struct Cpu {
        pc: BigEndian<u16>,

        /// General purpose registers.
        r0: BigEndian<u16>,
        r1: BigEndian<u16>,
        r2: BigEndian<u16>,
        r3: BigEndian<u16>,

        /// A simple zero flag.
        zf: u8,
    }

    impl Default for Cpu {
        fn default() -> Self {
            Self {
                pc: 0u16.into(),
                r0: 0u16.into(),
                r1: 0u16.into(),
                r2: 0u16.into(),
                r3: 0u16.into(),
                zf: 0,
            }
        }
    }

    /// Instruction encoding (2 bytes / 1 word):
    ///
    /// - Byte0: opcode
    /// - Byte1: operand (meaning depends on opcode)
    ///
    /// Opcodes:
    /// - 0x10..=0x13: LDI rX, imm8   (load imm8 into rX)
    /// - 0x20:        ADD r0, r1     (r0 = r0 + r1)
    /// - 0x21:        ADD r2, r3     (r2 = r2 + r3)
    /// - 0x30:        STORE r0, [addr]  (store r0 as BE u16 to memory[addr..addr+2])
    /// - 0x31:        LOAD  r2, [addr]  (load BE u16 from memory into r2)
    /// - 0x40:        CMP r0, r2     (zf = (r0 == r2))
    /// - 0x50:        JZ addr        (if zf==1, pc = addr)
    /// - 0xFF:        HALT
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Op {
        Ldi { reg: u8, imm: u8 },
        Add { lhs: u8, rhs: u8 },
        Store { reg: u8, addr: u8 },
        Load { reg: u8, addr: u8 },
        Cmp { a: u8, b: u8 },
        Jz { addr: u8 },
        Halt,
    }

    fn decode(word: [u8; 2]) -> Result<Op, String> {
        let op = word[0];
        let b = word[1];
        match op {
            0x10..=0x13 => Ok(Op::Ldi {
                reg: op - 0x10,
                imm: b,
            }),
            0x20 => Ok(Op::Add { lhs: 0, rhs: 1 }),
            0x21 => Ok(Op::Add { lhs: 2, rhs: 3 }),
            0x30 => Ok(Op::Store { reg: 0, addr: b }),
            0x31 => Ok(Op::Load { reg: 2, addr: b }),
            0x40 => Ok(Op::Cmp { a: 0, b: 2 }),
            0x50 => Ok(Op::Jz { addr: b }),
            0xFF => Ok(Op::Halt),
            _ => Err(format!("unknown opcode 0x{op:02X}")),
        }
    }

    impl Cpu {
        fn get_reg(&self, r: u8) -> BigEndian<u16> {
            match r {
                0 => self.r0,
                1 => self.r1,
                2 => self.r2,
                3 => self.r3,
                _ => 0u16.into(),
            }
        }

        fn set_reg(&mut self, r: u8, v: BigEndian<u16>) {
            match r {
                0 => self.r0 = v,
                1 => self.r1 = v,
                2 => self.r2 = v,
                3 => self.r3 = v,
                _ => {}
            }
        }

        fn fetch(&mut self, mem: &[u8]) -> Result<[u8; 2], String> {
            let pc = self.pc.to_native() as usize;
            if pc + 2 > mem.len() {
                return Err("pc out of bounds".into());
            }
            let w = [mem[pc], mem[pc + 1]];
            self.pc = (pc as u16).wrapping_add(2).into();
            Ok(w)
        }

        fn step(&mut self, mem: &mut [u8]) -> Result<bool, String> {
            let word = self.fetch(mem)?;
            let op = decode(word)?;

            match op {
                Op::Ldi { reg, imm } => {
                    self.set_reg(reg, (imm as u16).into());
                }
                Op::Add { lhs, rhs } => {
                    let a = self.get_reg(lhs).to_native();
                    let b = self.get_reg(rhs).to_native();
                    let sum = a.wrapping_add(b);
                    self.set_reg(lhs, sum.into());
                    self.zf = (sum == 0) as u8;
                }
                Op::Store { reg, addr } => {
                    let v = self.get_reg(reg);
                    let a = addr as usize;
                    if a + 2 > mem.len() {
                        return Err("store out of bounds".into());
                    }
                    // Store BE u16 to memory using crate IO.
                    let mut cur = Cursor::new(&mut mem[a..a + 2]);
                    let wire: u16be = v.to_native().into();
                    write_specific(&mut cur, &wire).map_err(|e| e.to_string())?;
                }
                Op::Load { reg, addr } => {
                    let a = addr as usize;
                    if a + 2 > mem.len() {
                        return Err("load out of bounds".into());
                    }
                    // Load BE u16 from memory into a BE register using crate IO.
                    let mut cur = Cursor::new(&mem[a..a + 2]);
                    let wire: u16be = read_specific(&mut cur).map_err(|e| e.to_string())?;
                    self.set_reg(reg, wire.to_native().into());
                }
                Op::Cmp { a, b } => {
                    let x = self.get_reg(a).to_native();
                    let y = self.get_reg(b).to_native();
                    self.zf = (x == y) as u8;
                }
                Op::Jz { addr } => {
                    if self.zf != 0 {
                        // addr is in bytes
                        self.pc = (addr as u16).into();
                    }
                }
                Op::Halt => return Ok(false),
            }

            Ok(true)
        }
    }

    pub fn run() {
        // A tiny program that demonstrates BE registers and BE memory operations:
        //
        // 0x0000: LDI r0, 0x01
        // 0x0002: LDI r1, 0x02
        // 0x0004: ADD r0, r1        ; r0 = 3
        // 0x0006: STORE r0, [0x10]  ; mem[0x10..0x12] = 00 03
        // 0x0008: LOAD  r2, [0x10]  ; r2 = 3
        // 0x000A: CMP r0, r2        ; zf = 1
        // 0x000C: JZ 0x0E           ; jump to HALT
        // 0x000E: HALT
        let program: [u8; 16] = [
            0x10, 0x01, // LDI r0, 1
            0x11, 0x02, // LDI r1, 2
            0x20, 0x00, // ADD r0, r1
            0x30, 0x10, // STORE r0, [0x10]
            0x31, 0x10, // LOAD r2, [0x10]
            0x40, 0x00, // CMP r0, r2
            0x50, 0x0E, // JZ 0x0E
            0xFF, 0x00, // HALT
        ];

        let mut mem = [0u8; 0x40];
        mem[0..program.len()].copy_from_slice(&program);

        let mut cpu = Cpu::default();

        // Run with a simple instruction limit to avoid accidental infinite loops.
        for _ in 0..128 {
            if !cpu.step(&mut mem).expect("cpu step") {
                break;
            }
        }

        let mem_word: u16 = {
            let mut cur = Cursor::new(&mem[0x10..0x12]);
            let wire: u16be = read_specific(&mut cur).expect("read u16be from memory");
            wire.to_native()
        };

        println!("Final CPU state: {cpu:?}");
        println!(
            "r0(native)={}, r2(native)={}, zf={}, mem[0x10..0x12]=[{:#04X} {:#04X}] (u16BE={})",
            cpu.r0.to_native(),
            cpu.r2.to_native(),
            cpu.zf,
            mem[0x10],
            mem[0x11],
            mem_word
        );
    }
}

#[cfg(feature = "io-std")]
fn main() {
    real::run();
}

#[cfg(not(feature = "io-std"))]
fn main() {
    eprintln!(
        "This example requires feature: io-std\n\n  cargo run --example cpu_emulator --features \"io-std\""
    );
}
