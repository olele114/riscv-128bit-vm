# RISC-V 128-bit Virtual Machine

A high-performance 128-bit RISC-V virtual machine implemented in Rust, featuring custom instruction set extensions and advanced memory management.

## Features

- **128-bit Registers**: Full support for RISC-V 128-bit integer extension (I)
- **32 Floating-Point Registers**: Support for F/D/Q floating-point extensions
- **32 Vector Registers**: Support for V vector extension
- **Built-in Assembler**: Direct loading and execution of RISC-V assembly code
- **Flexible Memory System**: Configurable memory size with byte/half-word/word/double-word/quad-word access
- **Interactive Debugger**: Full-featured debugger with breakpoints, watchpoints, execution history, and step-by-step execution
- **Debug Support**: Single-step execution, instruction tracing, register state inspection
- **Exception Handling**: Complete exception capture and reporting mechanism

## Supported Extensions

### Base Integer Instructions (I)

| Type | Instructions |
|------|--------------|
| R-type | ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND |
| I-type | ADDI, SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI, SRAI |
| Load | LB, LH, LW, LD, LQ, LBU, LHU, LWU, LDU |
| Store | SB, SH, SW, SD, SQ |
| Branch | BEQ, BNE, BLT, BGE, BLTU, BGEU |
| U-type | LUI, AUIPC |
| Jump | JAL, JALR |
| System | ECALL, EBREAK |

### M Extension (Integer Multiplication/Division)

| Instruction | Description |
|-------------|-------------|
| MUL | Multiply low |
| MULH | Multiply high signed |
| MULHSU | Multiply high signed-unsigned |
| MULHU | Multiply high unsigned |
| DIV | Divide signed |
| DIVU | Divide unsigned |
| REM | Remainder signed |
| REMU | Remainder unsigned |

### A Extension (Atomic Memory Operations)

| Instruction | Description |
|-------------|-------------|
| LR.D | Load Reserved |
| SC.D | Store Conditional |
| AMOADD.D | Atomic Add |
| AMOSWAP.D | Atomic Swap |
| AMOAND.D | Atomic AND |
| AMOOR.D | Atomic OR |
| AMOXOR.D | Atomic XOR |
| AMOMAX.D | Atomic Maximum signed |
| AMOMAXU.D | Atomic Maximum unsigned |
| AMOMIN.D | Atomic Minimum signed |
| AMOMINU.D | Atomic Minimum unsigned |

### F Extension (Single-Precision Floating-Point)

| Category | Instructions |
|----------|--------------|
| Load/Store | FLW, FSW |
| Arithmetic | FADD.S, FSUB.S, FMUL.S, FDIV.S, FSQRT.S |
| Sign Injection | FSGNJ.S, FSGNJN.S, FSGNJX.S |
| Min/Max | FMIN.S, FMAX.S |
| Convert | FCVT.W.S, FCVT.WU.S, FCVT.S.W, FCVT.S.WU, FCVT.L.S, FCVT.LU.S, FCVT.S.L, FCVT.S.LU |
| Move | FMV.X.S, FMV.S.X |
| Compare | FEQ.S, FLT.S, FLE.S |
| Classify | FCLASS.S |

### D Extension (Double-Precision Floating-Point)

| Category | Instructions |
|----------|--------------|
| Load/Store | FLD, FSD |
| Arithmetic | FADD.D, FSUB.D, FMUL.D, FDIV.D, FSQRT.D |
| Sign Injection | FSGNJ.D, FSGNJN.D, FSGNJX.D |
| Min/Max | FMIN.D, FMAX.D |
| Convert | FCVT.W.D, FCVT.WU.D, FCVT.D.W, FCVT.D.WU, FCVT.L.D, FCVT.LU.D, FCVT.D.L, FCVT.D.LU |
| Move | FMV.X.D, FMV.D.X |
| Compare | FEQ.D, FLT.D, FLE.D |
| Classify | FCLASS.D |
| Precision Convert | FCVT.D.S, FCVT.S.D |

### Q Extension (Quad-Precision Floating-Point)

| Category | Instructions |
|----------|--------------|
| Load/Store | FLQ, FSQ |
| Arithmetic | FADD.Q, FSUB.Q, FMUL.Q, FDIV.Q, FSQRT.Q |
| Sign Injection | FSGNJ.Q, FSGNJN.Q, FSGNJX.Q |
| Min/Max | FMIN.Q, FMAX.Q |
| Convert | FCVT.W.Q, FCVT.WU.Q, FCVT.Q.W, FCVT.Q.WU, FCVT.L.Q, FCVT.LU.Q, FCVT.Q.L, FCVT.Q.LU |
| Move | FMV.X.Q, FMV.Q.X |
| Compare | FEQ.Q, FLT.Q, FLE.Q |
| Classify | FCLASS.Q |
| Precision Convert | FCVT.Q.S, FCVT.S.Q, FCVT.Q.D, FCVT.D.Q |

### V Extension (Vector Operations)

- Vector load/store operations
- Vector integer operations
- Vector floating-point operations
- Vector configuration (vsetvli, vsetvl)
- Support for various element widths (8/16/32/64/128/256/512/1024-bit)
- Length multiplier (LMUL) support

### C Extension (Compressed Instructions)

16-bit compressed instructions for code size reduction:
- C.ADDI4SPN, C.LW, C.LD, C.SW, C.SD
- C.ADDI, C.JAL, C.LI, C.ADDI16SP, C.LUI
- C.SRLI, C.SRAI, C.ANDI, C.SUB, C.XOR, C.OR, C.AND
- C.J, C.BEQZ, C.BNEZ
- And more...

### Zicsr Extension (Control and Status Registers)

| Instruction | Description |
|-------------|-------------|
| CSRRW | CSR Read/Write |
| CSRRS | CSR Read/Set |
| CSRRC | CSR Read/Clear |
| CSRRWI | CSR Read/Write Immediate |
| CSRRSI | CSR Read/Set Immediate |
| CSRRCI | CSR Read/Clear Immediate |

### Zifencei Extension (Instruction Fence)

| Instruction | Description |
|-------------|-------------|
| FENCE.I | Instruction cache synchronization |

### Zba/Zbb/Zbc Extensions (Bit Manipulation)

| Category | Instructions |
|----------|--------------|
| Shift-Add | SH1ADD, SH2ADD, SH3ADD, SH1ADD.UW, SH2ADD.UW, SH3ADD.UW |
| Rotate | ROL, ROR, RORI |
| Logic | ANDN, ORN, XORN |
| Count | CLZ, CTZ, CPOP |
| Carry-less | CLMUL, CLMULR, CLMULH |
| Min/Max | MIN, MAX, MINU, MAXU |
| Bit Extract/Insert | BEXTI, BCLRI, BSETI, BINVI |
| Special | ADD.UW, SLLI.UW |

### Pseudo-Instructions

The assembler supports the following pseudo-instructions for simplified programming:

```
nop         ->  addi x0, x0, 0
mv rd, rs   ->  addi rd, rs, 0
not rd, rs  ->  xori rd, rs, -1
neg rd, rs  ->  sub rd, x0, rs
li rd, imm  ->  lui + addi (automatically expanded based on immediate)
j label     ->  jal x0, label
ret         ->  jalr x0, x1, 0
call label  ->  auipc + jalr
beqz rs, label -> beq rs, x0, label
...and more
```

## Installation

### Prerequisites

- Rust 1.70 or higher

### Building

```bash
git clone https://github.com/olele114/riscv-128bit-vm.git
cd riscv-128bit-vm
cargo build --release
```

The compiled executable will be located at `target/release/riscv-128bit-vm.exe`.

## Usage

### Command Line Arguments

```
riscv-128bit-vm [options] [program_file]

Options:
  --help              Show help message
  --memory <size>     Set memory size in bytes (default: 16MB)
  --debug             Enable debug mode
  --trace             Enable execution tracing
  --step              Run in single-step mode
  -d, --debugger      Start interactive debugger
  --history <n>       Set execution history size (default: 1000)
  --load-addr <addr>  Set program load address (default: 0x0)
```

### Supported File Formats

| Extension | Format |
|-----------|--------|
| .bin, .raw | Raw binary machine code |
| .s, .asm | RISC-V assembly source (auto-assembled) |

### Interactive Debugger

The interactive debugger provides powerful debugging capabilities:

```
Debugger Commands:
  Execution Control:
    c, continue       Continue execution
    s, step           Single step (enter function calls)
    n, next           Step over (skip function calls)
    finish            Step out of current function
    u, until <addr>   Run until address

  Breakpoints:
    b, break <addr>   Set breakpoint at address
    tb, tbreak <addr> Set temporary breakpoint (removed after hit)
    d, delete [id]    Delete breakpoint (all if no id)
    enable <id>       Enable breakpoint
    disable <id>      Disable breakpoint
    ignore <id> <n>   Ignore breakpoint for N hits
    info b            List all breakpoints

  Watchpoints:
    watch <addr> [size] [type]  Set memory watchpoint
                                type: r(ead), w(rite), a(ccess)
    watchreg <reg> [fp]         Watch register (fp for float)
    dwatch <id>                 Delete watchpoint
    info w                      List all watchpoints

  Inspection:
    p, print <$reg>   Print register value
    p, print <addr> [size]  Print memory contents
    reg, registers    Print all registers
    x <addr> [size]   Examine memory
    disas <addr> [n]  Disassemble N instructions

  History & Navigation:
    history [n]       Show last N execution history entries
    where             Show current position and instruction
    reset             Reset VM to initial state

  Other:
    help, ?           Show help
    q, quit           Exit debugger
```

### Examples

```bash
# Run a binary program
riscv-128bit-vm program.bin

# Run an assembly program
riscv-128bit-vm program.s

# Run with specified load address
riscv-128bit-vm --load-addr 0x80000000 firmware.asm

# Run in debug mode
riscv-128bit-vm --debug --trace program.s

# Single-step execution
riscv-128bit-vm --step program.s

# Start interactive debugger
riscv-128bit-vm -d program.s

# Custom memory size (32MB) with debugger
riscv-128bit-vm --memory 0x2000000 -d program.bin

# Set history size for debugger
riscv-128bit-vm --history 5000 -d program.s
```

## Assembly Language Examples

### Basic Computation

```asm
# Simple calculation example
.text

    li a0, 42          # Load immediate 42 into a0
    li a1, 10          # Load immediate 10 into a1
    add a2, a0, a1     # a2 = a0 + a1 (result: 52)
    sub a3, a0, a1     # a3 = a0 - a1 (result: 32)
    
    ebreak             # Stop execution
```

### Loop Example

```asm
# Calculate 1+2+3+4+5 = 15
.text

    li t0, 0           # Accumulator
    li t1, 1           # Counter
    li t2, 6           # End condition

loop:
    add t0, t0, t1     # Accumulate
    addi t1, t1, 1     # Counter +1
    blt t1, t2, loop   # If t1 < 6, continue loop
    
    mv a0, t0          # Store result in a0 (15)
    ebreak
```

### Memory Operations

```asm
# Data storage and loading
.text

    li t0, 0x1000      # Memory address
    
    li t1, 0x12345678  # Test data
    sw t1, 0(t0)       # Store word
    
    lw t2, 0(t0)       # Load word
    
    addi t3, t2, 1     # t3 = t2 + 1
    
    ebreak
```

### Floating-Point Example

```asm
# Floating-point computation
.text

    li t0, 0x1000      # Memory address for FP data
    
    # Store and load single-precision
    flw f0, 0(t0)      # Load single-precision
    fadd.s f2, f0, f1  # Add single-precision
    
    # Store and load double-precision
    fld f4, 8(t0)      # Load double-precision
    fmul.d f6, f4, f5  # Multiply double-precision
    
    ebreak
```

### Atomic Operations Example

```asm
# Atomic memory operations
.text

    li t0, 0x1000      # Memory address
    
    lr.d t1, 0(t0)     # Load reserved
    addi t1, t1, 1     # Increment
    sc.d t2, t1, 0(t0) # Store conditional
    
    beqz t2, done      # If success (t2 == 0), done
    j atomic_retry     # Retry on failure
    
done:
    ebreak
```

## Programming Interface

### Basic Usage

```rust
use riscv::virtual_machine::{VirtualMachine, VMConfig};
use riscv::memory;

fn main() {
    // Create configuration
    let config = VMConfig::new();
    
    // Create virtual machine
    let mut vm = VirtualMachine::new(config);
    vm.initialize();
    
    // Load program
    vm.load_program("program.bin", 0x0).unwrap();
    
    // Run
    vm.run();
    
    // View state
    vm.print_register_state();
}
```

### Loading Assembly Code

```rust
// Load assembly from file
vm.load_assembly("program.s", 0x0).unwrap();

// Load assembly from string
let source = r#"
    li a0, 42
    add a1, a0, a0
    ebreak
"#;
vm.load_assembly_string(source, 0x0).unwrap();
```

### Single-Step Execution

```rust
vm.start();  // Start CPU

while vm.is_running() && !vm.has_exception() {
    vm.step();  // Execute one instruction
    
    // View state
    vm.print_register_state();
}
```

### Interactive Debugger

```rust
use riscv::virtual_machine::{VirtualMachine, VMConfig};

// Create VM with debugger enabled
let config = VMConfig::new().with_debugger().with_history_size(2000);
let mut vm = VirtualMachine::new(config);
vm.initialize();

// Load program
vm.load_assembly("program.s", 0x0).unwrap();

// Run interactive debugger
vm.run_debugger();
```

### Debugger API

```rust
// Programmatic debugger usage
if let Some(debugger) = vm.get_debugger_mut() {
    // Add breakpoint
    let bp_id = debugger.breakpoints.add_address(0x80000100);
    
    // Add conditional breakpoint
    let cond = debugger::BreakCondition::RegisterEqual { reg: 10, value: 42 };
    debugger.breakpoints.add_conditional(0x80000200, cond);
    
    // Add watchpoint
    debugger.watchpoints.add_memory(0x1000, 8, debugger::WatchpointType::Write);
}

// Check and manage breakpoints programmatically
if let Some(debugger) = vm.get_debugger() {
    for bp in debugger.breakpoints.list() {
        println!("Breakpoint {} at 0x{:x}, hits: {}", 
                 bp.id, bp.address().unwrap_or(0), bp.hit_count);
    }
}
```

## Project Structure

```
riscv-128bit-vm/
├── Cargo.toml
└── src/
    ├── main.rs              # Main program entry
    └── riscv/
        ├── mod.rs           # Module exports
        ├── cpu.rs           # CPU implementation
        ├── register.rs      # Register file (GPR, FPR, VR, CSR)
        ├── memory.rs        # Memory system with AMO support
        ├── instruction.rs   # Instruction decoder
        ├── executor.rs      # Instruction executor
        ├── virtual_machine.rs # VM interface
        ├── assembler.rs     # Assembler
        └── debugger.rs      # Interactive debugger
```

## Register Mapping

### General-Purpose Registers

| Register | ABI Name | Purpose |
|----------|----------|---------|
| x0 | zero | Hardwired zero |
| x1 | ra | Return address |
| x2 | sp | Stack pointer |
| x3 | gp | Global pointer |
| x4 | tp | Thread pointer |
| x5-x7 | t0-t2 | Temporary registers |
| x8 | s0/fp | Saved register / Frame pointer |
| x9 | s1 | Saved register |
| x10-x17 | a0-a7 | Function arguments / Return values |
| x18-x27 | s2-s11 | Saved registers |
| x28-x31 | t3-t6 | Temporary registers |

### Floating-Point Registers

| Register | ABI Name | Purpose |
|----------|----------|---------|
| f0-f7 | ft0-ft7 | FP temporary registers |
| f8-f9 | fs0-fs1 | FP saved registers |
| f10-f17 | fa0-fa7 | FP arguments / Return values |
| f18-f27 | fs2-fs11 | FP saved registers |
| f28-f31 | ft8-ft11 | FP temporary registers |

### Vector Registers

| Register | ABI Name | Purpose |
|----------|----------|---------|
| v0-v31 | v0-v31 | Vector registers |

## License

MIT License