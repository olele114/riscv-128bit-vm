# RISC-V 128位虚拟机

一个高性能的 Rust 实现的 128 位 RISC-V 虚拟机，支持自定义指令集扩展和高级内存管理。

## 特性

- **128位寄存器**: 完整支持 RISC-V 128位整数扩展 (I)
- **32个浮点寄存器**: 支持 F/D/Q 浮点扩展
- **32个向量寄存器**: 支持 V 向量扩展
- **内置汇编器**: 支持直接加载和运行 RISC-V 汇编代码
- **灵活的内存系统**: 可配置内存大小，支持字节/半字/字/双字/四字访问
- **交互式调试器**: 功能完整的调试器，支持断点、观察点、执行历史、单步执行
- **调试支持**: 单步执行、指令追踪、寄存器状态查看
- **异常处理**: 完整的异常捕获和报告机制

## 支持的扩展

### 基础整数指令 (I)

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

### M 扩展 (整数乘除法)

| 指令 | 描述 |
|------|------|
| MUL | 乘法低位 |
| MULH | 乘法高位 (有符号) |
| MULHSU | 乘法高位 (有符号-无符号) |
| MULHU | 乘法高位 (无符号) |
| DIV | 除法 (有符号) |
| DIVU | 除法 (无符号) |
| REM | 余数 (有符号) |
| REMU | 余数 (无符号) |

### A 扩展 (原子内存操作)

| 指令 | 描述 |
|------|------|
| LR.D | 加载保留 |
| SC.D | 存储条件 |
| AMOADD.D | 原子加法 |
| AMOSWAP.D | 原子交换 |
| AMOAND.D | 原子与 |
| AMOOR.D | 原子或 |
| AMOXOR.D | 原子异或 |
| AMOMAX.D | 原子最大值 (有符号) |
| AMOMAXU.D | 原子最大值 (无符号) |
| AMOMIN.D | 原子最小值 (有符号) |
| AMOMINU.D | 原子最小值 (无符号) |

### F 扩展 (单精度浮点)

| 类别 | 指令 |
|------|------|
| 加载/存储 | FLW, FSW |
| 算术 | FADD.S, FSUB.S, FMUL.S, FDIV.S, FSQRT.S |
| 符号注入 | FSGNJ.S, FSGNJN.S, FSGNJX.S |
| 最小/最大 | FMIN.S, FMAX.S |
| 转换 | FCVT.W.S, FCVT.WU.S, FCVT.S.W, FCVT.S.WU, FCVT.L.S, FCVT.LU.S, FCVT.S.L, FCVT.S.LU |
| 移动 | FMV.X.S, FMV.S.X |
| 比较 | FEQ.S, FLT.S, FLE.S |
| 分类 | FCLASS.S |

### D 扩展 (双精度浮点)

| 类别 | 指令 |
|------|------|
| 加载/存储 | FLD, FSD |
| 算术 | FADD.D, FSUB.D, FMUL.D, FDIV.D, FSQRT.D |
| 符号注入 | FSGNJ.D, FSGNJN.D, FSGNJX.D |
| 最小/最大 | FMIN.D, FMAX.D |
| 转换 | FCVT.W.D, FCVT.WU.D, FCVT.D.W, FCVT.D.WU, FCVT.L.D, FCVT.LU.D, FCVT.D.L, FCVT.D.LU |
| 移动 | FMV.X.D, FMV.D.X |
| 比较 | FEQ.D, FLT.D, FLE.D |
| 分类 | FCLASS.D |
| 精度转换 | FCVT.D.S, FCVT.S.D |

### Q 扩展 (四精度浮点)

| 类别 | 指令 |
|------|------|
| 加载/存储 | FLQ, FSQ |
| 算术 | FADD.Q, FSUB.Q, FMUL.Q, FDIV.Q, FSQRT.Q |
| 符号注入 | FSGNJ.Q, FSGNJN.Q, FSGNJX.Q |
| 最小/最大 | FMIN.Q, FMAX.Q |
| 转换 | FCVT.W.Q, FCVT.WU.Q, FCVT.Q.W, FCVT.Q.WU, FCVT.L.Q, FCVT.LU.Q, FCVT.Q.L, FCVT.Q.LU |
| 移动 | FMV.X.Q, FMV.Q.X |
| 比较 | FEQ.Q, FLT.Q, FLE.Q |
| 分类 | FCLASS.Q |
| 精度转换 | FCVT.Q.S, FCVT.S.Q, FCVT.Q.D, FCVT.D.Q |

### V 扩展 (向量操作)

- 向量加载/存储操作
- 向量整数操作
- 向量浮点操作
- 向量配置 (vsetvli, vsetvl)
- 支持多种元素宽度 (8/16/32/64/128/256/512/1024位)
- 长度乘数 (LMUL) 支持

### C 扩展 (压缩指令)

16位压缩指令，用于减小代码体积：
- C.ADDI4SPN, C.LW, C.LD, C.SW, C.SD
- C.ADDI, C.JAL, C.LI, C.ADDI16SP, C.LUI
- C.SRLI, C.SRAI, C.ANDI, C.SUB, C.XOR, C.OR, C.AND
- C.J, C.BEQZ, C.BNEZ
- 更多...

### Zicsr 扩展 (控制状态寄存器)

| 指令 | 描述 |
|------|------|
| CSRRW | CSR 读写 |
| CSRRS | CSR 读置位 |
| CSRRC | CSR 读清除 |
| CSRRWI | CSR 读写立即数 |
| CSRRSI | CSR 读置位立即数 |
| CSRRCI | CSR 读清除立即数 |

### Zifencei 扩展 (指令屏障)

| 指令 | 描述 |
|------|------|
| FENCE.I | 指令缓存同步 |

### Zba/Zbb/Zbc 扩展 (位操作)

| 类别 | 指令 |
|------|------|
| 移位加法 | SH1ADD, SH2ADD, SH3ADD, SH1ADD.UW, SH2ADD.UW, SH3ADD.UW |
| 循环移位 | ROL, ROR, RORI |
| 逻辑运算 | ANDN, ORN, XORN |
| 计数 | CLZ, CTZ, CPOP |
| 无进位乘法 | CLMUL, CLMULR, CLMULH |
| 最小/最大 | MIN, MAX, MINU, MAXU |
| 位提取/插入 | BEXTI, BCLRI, BSETI, BINVI |
| 特殊操作 | ADD.UW, SLLI.UW |

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
git clone https://github.com/olele114/riscv-128bit-vm.git
cd riscv-128bit-vm
cargo build --release
```

编译后的可执行文件位于 `target/release/riscv-128bit-vm.exe`。

## 使用方法

### 命令行参数

```
riscv-128bit-vm [options] [program_file]

选项:
  --help              显示帮助信息
  --memory <size>     设置内存大小 (默认: 16MB)
  --debug             启用调试模式
  --trace             启用执行追踪
  --step              单步执行模式
  -d, --debugger      启动交互式调试器
  --history <n>       设置执行历史大小 (默认: 1000)
  --load-addr <addr>  设置程序加载地址 (默认: 0x0)
```

### 支持的文件格式

| 扩展名 | 格式 |
|--------|------|
| .bin, .raw | 原始二进制机器码 |
| .s, .asm | RISC-V 汇编源码 (自动汇编) |

### 交互式调试器

交互式调试器提供强大的调试功能：

```
调试器命令:
  执行控制:
    c, continue       继续执行
    s, step           单步执行 (进入函数调用)
    n, next           步过 (跳过函数调用)
    finish            步出当前函数
    u, until <addr>   运行到指定地址

  断点:
    b, break <addr>   在地址设置断点
    tb, tbreak <addr> 设置临时断点 (触发后删除)
    d, delete [id]    删除断点 (无 id 时删除全部)
    enable <id>       启用断点
    disable <id>      禁用断点
    ignore <id> <n>   忽略断点 N 次命中
    info b            列出所有断点

  观察点:
    watch <addr> [size] [type]  设置内存观察点
                                type: r(读), w(写), a(访问)
    watchreg <reg> [fp]         监视寄存器 (fp 表示浮点寄存器)
    dwatch <id>                 删除观察点
    info w                      列出所有观察点

  检查:
    p, print <$reg>   打印寄存器值
    p, print <addr> [size]  打印内存内容
    reg, registers    打印所有寄存器
    x <addr> [size]   检查内存
    disas <addr> [n]  反汇编 N 条指令

  历史与导航:
    history [n]       显示最近 N 条执行历史
    where             显示当前位置和指令
    reset             重置虚拟机到初始状态

  其他:
    help, ?           显示帮助
    q, quit           退出调试器
```

### 示例

```bash
# 运行二进制程序
riscv-128bit-vm program.bin

# 运行汇编程序
riscv-128bit-vm program.s

# 指定加载地址运行
riscv-128bit-vm --load-addr 0x80000000 firmware.asm

# 调试模式运行
riscv-128bit-vm --debug --trace program.s

# 单步执行
riscv-128bit-vm --step program.s

# 启动交互式调试器
riscv-128bit-vm -d program.s

# 自定义内存大小 (32MB) 并启动调试器
riscv-128bit-vm --memory 0x2000000 -d program.bin

# 设置调试器历史大小
riscv-128bit-vm --history 5000 -d program.s
```

## 汇编语言示例

### 基本计算

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

### 浮点运算示例

```asm
# 浮点计算
.text

    li t0, 0x1000      # 浮点数据内存地址
    
    # 单精度加载和存储
    flw f0, 0(t0)      # 加载单精度浮点
    fadd.s f2, f0, f1  # 单精度加法
    
    # 双精度加载和存储
    fld f4, 8(t0)      # 加载双精度浮点
    fmul.d f6, f4, f5  # 双精度乘法
    
    ebreak
```

### 原子操作示例

```asm
# 原子内存操作
.text

    li t0, 0x1000      # 内存地址
    
    lr.d t1, 0(t0)     # 加载保留
    addi t1, t1, 1     # 自增
    sc.d t2, t1, 0(t0) # 存储条件
    
    beqz t2, done      # 如果成功 (t2 == 0), 完成
    j atomic_retry     # 失败则重试
    
done:
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

### 交互式调试器

```rust
use riscv::virtual_machine::{VirtualMachine, VMConfig};

// 创建启用调试器的虚拟机
let config = VMConfig::new().with_debugger().with_history_size(2000);
let mut vm = VirtualMachine::new(config);
vm.initialize();

// 加载程序
vm.load_assembly("program.s", 0x0).unwrap();

// 运行交互式调试器
vm.run_debugger();
```

### 调试器 API

```rust
// 编程式使用调试器
if let Some(debugger) = vm.get_debugger_mut() {
    // 添加断点
    let bp_id = debugger.breakpoints.add_address(0x80000100);
    
    // 添加条件断点
    let cond = debugger::BreakCondition::RegisterEqual { reg: 10, value: 42 };
    debugger.breakpoints.add_conditional(0x80000200, cond);
    
    // 添加观察点
    debugger.watchpoints.add_memory(0x1000, 8, debugger::WatchpointType::Write);
}

// 编程式检查和管理断点
if let Some(debugger) = vm.get_debugger() {
    for bp in debugger.breakpoints.list() {
        println!("断点 {} 在 0x{:x}, 命中次数: {}", 
                 bp.id, bp.address().unwrap_or(0), bp.hit_count);
    }
}
```

## 项目结构

```
riscv-128bit-vm/
├── Cargo.toml
└── src/
    ├── main.rs              # 主程序入口
    └── riscv/
        ├── mod.rs           # 模块导出
        ├── cpu.rs           # CPU 实现
        ├── register.rs      # 寄存器组 (GPR, FPR, VR, CSR)
        ├── memory.rs        # 内存系统，支持 AMO
        ├── instruction.rs   # 指令解码器
        ├── executor.rs      # 指令执行器
        ├── virtual_machine.rs # 虚拟机接口
        ├── assembler.rs     # 汇编器
        └── debugger.rs      # 交互式调试器
```

## 寄存器映射

### 通用寄存器

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

### 浮点寄存器

| 寄存器 | ABI 名称 | 用途 |
|--------|----------|------|
| f0-f7 | ft0-ft7 | 浮点临时寄存器 |
| f8-f9 | fs0-fs1 | 浮点保存寄存器 |
| f10-f17 | fa0-fa7 | 浮点参数/返回值 |
| f18-f27 | fs2-fs11 | 浮点保存寄存器 |
| f28-f31 | ft8-ft11 | 浮点临时寄存器 |

### 向量寄存器

| 寄存器 | ABI 名称 | 用途 |
|--------|----------|------|
| v0-v31 | v0-v31 | 向量寄存器 |

## 许可证

MIT License