# RISC-V 128-bit Virtual Machine

A RISC-V 128-bit virtual machine framework implemented in Rust, supporting direct machine code execution and just-in-time assembly.

## Features

- **128-bit Registers**: Full support for RISC-V 128-bit integer extension
- **Built-in Assembler**: Direct loading and execution of RISC-V assembly code
- **Flexible Memory System**: Configurable memory size with byte/half-word/word/double-word/quad-word access
- **Debug Support**: Single-step execution, instruction tracing, register state inspection
- **Exception Handling**: Complete exception capture and reporting mechanism

## Supported Instruction Set

### Base Integer Instructions

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
git clone <repository-url>
cd vm_project
cargo build --release
```

The compiled executable will be located at `target/release/vm_project.exe`.

## Usage

### Command Line Arguments

```
vm_project [options] [program_file]

Options:
  --help              Show help message
  --memory <size>     Set memory size in bytes (default: 16MB)
  --debug             Enable debug mode
  --trace             Enable execution tracing
  --step              Run in single-step mode
  --load-addr <addr>  Set program load address (default: 0x0)
```

### Supported File Formats

| Extension | Format |
|-----------|--------|
| .bin, .raw | Raw binary machine code |
| .s, .asm | RISC-V assembly source (auto-assembled) |

### Examples

```bash
# Run a binary program
vm_project program.bin

# Run an assembly program
vm_project program.s

# Run with specified load address
vm_project --load-addr 0x80000000 firmware.asm

# Run in debug mode
vm_project --debug --trace program.s

# Single-step execution
vm_project --step program.s

# Custom memory size (32MB)
vm_project --memory 0x2000000 program.bin
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

## Project Structure

```
vm_project/
├── Cargo.toml
└── src/
    ├── main.rs              # Main program entry
    └── riscv/
        ├── mod.rs           # Module exports
        ├── cpu.rs           # CPU implementation
        ├── register.rs      # Register implementation
        ├── memory.rs        # Memory implementation
        ├── instruction.rs   # Instruction decoder
        ├── executor.rs      # Instruction executor
        ├── virtual_machine.rs # VM interface
        └── assembler.rs     # Assembler
```

## Register Mapping

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

## License

MIT License
