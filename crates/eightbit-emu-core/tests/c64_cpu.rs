use eightbit_emu_core::{
    CoreError,
    c64::{Bus, Cpu, reset, step, trigger_irq, trigger_nmi},
};

struct FakeBus {
    mem: Vec<u8>,
}

impl FakeBus {
    fn new() -> Self {
        Self {
            mem: vec![0; 65536],
        }
    }

    fn load(&mut self, addr: u16, bytes: &[u8]) {
        for (offset, byte) in bytes.iter().enumerate() {
            self.mem[addr as usize + offset] = *byte;
        }
    }

    fn byte(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }
}

impl Bus for FakeBus {
    fn read(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.mem[addr as usize] = value;
    }
}

#[test]
fn lda_imm_and_sta_abs() {
    let mut cpu = Cpu::default();
    let mut bus = FakeBus::new();
    bus.load(0, &[0xA9, 0x42, 0x8D, 0x00, 0x20]);
    cpu.pc = 0;
    step(&mut cpu, &mut bus).expect("lda");
    assert_eq!(cpu.a, 0x42);
    step(&mut cpu, &mut bus).expect("sta");
    assert_eq!(bus.byte(0x2000), 0x42);
}

#[test]
fn decimal_adc_nine_plus_one() {
    let mut cpu = Cpu::default();
    let mut bus = FakeBus::new();
    bus.load(0, &[0xF8, 0xA9, 0x09, 0x69, 0x01]);
    cpu.pc = 0;
    step(&mut cpu, &mut bus).expect("sed");
    step(&mut cpu, &mut bus).expect("lda");
    step(&mut cpu, &mut bus).expect("adc");
    assert_eq!(cpu.a, 0x10);
}

#[test]
fn illegal_opcode_returns_error() {
    let mut cpu = Cpu::default();
    let mut bus = FakeBus::new();
    bus.mem[0] = 0x02;
    cpu.pc = 0;
    let err = step(&mut cpu, &mut bus).unwrap_err();
    assert_eq!(err, CoreError::Opcode { opcode: 0x02 });
}

#[test]
fn jsr_and_rts() {
    let mut cpu = Cpu::default();
    let mut bus = FakeBus::new();
    bus.load(0, &[0x20, 0x10, 0x00]);
    bus.mem[0x0010] = 0x60;
    cpu.pc = 0;
    step(&mut cpu, &mut bus).expect("jsr");
    assert_eq!(cpu.pc, 0x0010);
    step(&mut cpu, &mut bus).expect("rts");
    assert_eq!(cpu.pc, 0x0003);
}

#[test]
fn bne_not_taken_skips_offset() {
    let mut cpu = Cpu::default();
    let mut bus = FakeBus::new();
    bus.load(0, &[0xA9, 0x00, 0xD0, 0xF4, 0xA9, 0x55]);
    cpu.pc = 0;
    step(&mut cpu, &mut bus).expect("lda");
    step(&mut cpu, &mut bus).expect("bne");
    step(&mut cpu, &mut bus).expect("lda2");
    assert_eq!(cpu.a, 0x55);
}

#[test]
fn reset_and_interrupts_follow_the_vectors() {
    let mut cpu = Cpu::default();
    let mut bus = FakeBus::new();
    bus.mem[0xFFFC] = 0x00;
    bus.mem[0xFFFD] = 0x80;
    bus.mem[0xFFFE] = 0x00;
    bus.mem[0xFFFF] = 0x90;
    bus.mem[0xFFFA] = 0x00;
    bus.mem[0xFFFB] = 0xA0;
    bus.mem[0x8000] = 0xEA;
    bus.mem[0x9000] = 0x40;
    bus.mem[0xA000] = 0x40;
    reset(&mut cpu, &mut bus);
    assert_eq!(cpu.pc, 0x8000);
    cpu.status &= !0x04;
    trigger_irq(&mut cpu);
    step(&mut cpu, &mut bus).expect("irq");
    assert_eq!(cpu.pc, 0x9000);
    step(&mut cpu, &mut bus).expect("rti");
    trigger_nmi(&mut cpu);
    step(&mut cpu, &mut bus).expect("nmi");
    assert_eq!(cpu.pc, 0xA000);
}

#[test]
fn every_legal_opcode_executes_or_rejects() {
    let mut opcode = 0u16;
    while opcode < 256 {
        let byte = opcode as u8;
        let mut cpu = Cpu::default();
        let mut bus = FakeBus::new();
        bus.mem.fill(0xEA);
        bus.mem[0] = byte;
        bus.mem[1] = 0x00;
        bus.mem[2] = 0x10;
        bus.mem[0x1000] = 0x60;
        cpu.pc = 0;
        cpu.sp = 0xFD;
        match step(&mut cpu, &mut bus) {
            Ok(_) => {}
            Err(CoreError::Opcode { opcode: hit }) if hit == byte => {}
            Err(error) => panic!("opcode {byte:#04x}: {error:?}"),
        }
        opcode += 1;
    }
}
