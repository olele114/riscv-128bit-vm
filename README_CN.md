# RISC-V 128-bit Virtual Machine

一个用 Rust 实现的 RISC-V 128位虚拟机框架，支持机器码直接执行和汇编语言即时汇编运行。

## 特性

- **128位寄存器**: 完整支持 RISC-V 128位整数扩展
- **内置汇编器**: 支持直接加载和运行 RISC-V 汇编代码
- **灵活的内存系统**: 可配置内存大小，支持字节/半字/字/双字/四字访问
- **调试支持**: 单步执行、指令追踪、寄存器状态查看
- **异常处理**: 完整的异常捕获和报告机制

## 支持的指令集

### 基础整数指令

| 类型 | 指令 |
|------|------|
| R-type | ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND |
| I-type | ADDI, SLTI, SLTIU, XORI, ORI, ANDI, SLLI, SRLI, SRAI |
| Load | LB, LH, LW, LD, LQ, LBU, LHU, LWU, LDU |
| Store | SB, SH, SW, SD, SQ |
| Branch | BEQ, BNE, BLT, BGE, BLTU, BGEU |
| U-type | LUI, AUIPC |
| Jump | JAL, JALR |
| System | ECALL, EBREAK |

### 伪指令支持

汇编器支持以下伪指令以简化编程：

```
nop         ->  addi x0, x0, 0
mv rd, rs   ->  addi rd, rs, 0
not rd, rs  ->  xori rd, rs, -1
neg rd, rs  ->  sub rd, x0, rs
li rd, imm  ->  lui + addi (根据立即数自动展开)
j label     ->  jal x0, label
ret         ->  jalr x0, x1, 0
call label  ->  auipc + jalr
beqz rs, label -> beq rs, x0, label
...等
```

## 安装

### 前置要求

- Rust 1.70 或更高版本

### 编译

```bash
git clone <repository-url>
cd vm_project
cargo build --release
```

编译后的可执行文件位于 `target/release/vm_project.exe`。

## 使用方法

### 命令行参数

```
vm_project [options] [program_file]

选项:
  --help              显示帮助信息
  --memory <size>     设置内存大小 (默认: 16MB)
  --debug             启用调试模式
  --trace             启用执行追踪
  --step              单步执行模式
  --load-addr <addr>  设置程序加载地址 (默认: 0x0)
```

### 支持的文件格式

| 扩展名 | 格式 |
|--------|------|
| .bin, .raw | 原始二进制机器码 |
| .s, .asm | RISC-V 汇编源码 (自动汇编) |

### 示例

```bash
# 运行二进制程序
vm_project program.bin

# 运行汇编程序
vm_project program.s

# 指定加载地址运行
vm_project --load-addr 0x80000000 firmware.asm

# 调试模式运行
vm_project --debug --trace program.s

# 单步执行
vm_project --step program.s

# 自定义内存大小 (32MB)
vm_project --memory 0x2000000 program.bin
```

## 汇编语言示例

### Hello World 风格示例

```asm
# 简单计算示例
.text

    li a0, 42          # 加载立即数 42 到 a0
    li a1, 10          # 加载立即数 10 到 a1
    add a2, a0, a1     # a2 = a0 + a1 (结果: 52)
    sub a3, a0, a1     # a3 = a0 - a1 (结果: 32)
    
    ebreak             # 停止执行
```

### 循环示例

```asm
# 计算 1+2+3+4+5 = 15
.text

    li t0, 0           # 累加器
    li t1, 1           # 计数器
    li t2, 6           # 结束条件

loop:
    add t0, t0, t1     # 累加
    addi t1, t1, 1     # 计数器 +1
    blt t1, t2, loop   # 如果 t1 < 6, 继续循环
    
    mv a0, t0          # 结果存入 a0 (15)
    ebreak
```

### 内存操作示例

```asm
# 数据存储和加载
.text

    li t0, 0x1000      # 内存地址
    
    li t1, 0x12345678  # 测试数据
    sw t1, 0(t0)       # 存储字
    
    lw t2, 0(t0)       # 加载字
    
    addi t3, t2, 1     # t3 = t2 + 1
    
    ebreak
```

## 编程接口

### 基本使用

```rust
use riscv::virtual_machine::{VirtualMachine, VMConfig};
use riscv::memory;

fn main() {
    // 创建配置
    let config = VMConfig::new();
    
    // 创建虚拟机
    let mut vm = VirtualMachine::new(config);
    vm.initialize();
    
    // 加载程序
    vm.load_program("program.bin", 0x0).unwrap();
    
    // 运行
    vm.run();
    
    // 查看状态
    vm.print_register_state();
}
```

### 加载汇编代码

```rust
// 从文件加载汇编
vm.load_assembly("program.s", 0x0).unwrap();

// 从字符串加载汇编
let source = r#"
    li a0, 42
    add a1, a0, a0
    ebreak
"#;
vm.load_assembly_string(source, 0x0).unwrap();
```

### 单步执行

```rust
vm.start();  // 启动 CPU

while vm.is_running() && !vm.has_exception() {
    vm.step();  // 执行一条指令
    
    // 查看状态
    vm.print_register_state();
}
```

## 项目结构

```
vm_project/
├── Cargo.toml
└── src/
    ├── main.rs              # 主程序入口
    └── riscv/
        ├── mod.rs           # 模块导出
        ├── cpu.rs           # CPU 实现
        ├── register.rs      # 寄存器实现
        ├── memory.rs        # 内存实现
        ├── instruction.rs   # 指令解码器
        ├── executor.rs      # 指令执行器
        ├── virtual_machine.rs # 虚拟机接口
        └── assembler.rs     # 汇编器
```

## 寄存器映射

| 寄存器 | ABI 名称 | 用途 |
|--------|----------|------|
| x0 | zero | 硬件零 |
| x1 | ra | 返回地址 |
| x2 | sp | 栈指针 |
| x3 | gp | 全局指针 |
| x4 | tp | 线程指针 |
| x5-x7 | t0-t2 | 临时寄存器 |
| x8 | s0/fp | 保存寄存器/帧指针 |
| x9 | s1 | 保存寄存器 |
| x10-x17 | a0-a7 | 函数参数/返回值 |
| x18-x27 | s2-s11 | 保存寄存器 |
| x28-x31 | t3-t6 | 临时寄存器 |

## 许可证

MIT License
