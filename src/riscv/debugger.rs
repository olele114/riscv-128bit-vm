//! Debugger Module
//!
//! Provides comprehensive debugging capabilities for the RISC-V virtual machine:
//! - Breakpoints (address and conditional)
//! - Watchpoints (memory and register monitoring)
//! - Execution history recording and playback
//! - Interactive command-line debugger interface
//!
//! ---
//!
//! 调试器模块
//!
//! 为 RISC-V 虚拟机提供全面的调试功能：
//! - 断点（地址断点和条件断点）
//! - 观察点（内存和寄存器监视）
//! - 执行历史记录和回放
//! - 交互式命令行调试器界面

use crate::riscv::memory;
use crate::riscv::register;
use crate::riscv::assembler;
use std::collections::HashMap;

// ========================================
// Breakpoint System / 断点系统
// ========================================

/// Breakpoint identifier type.
///
/// ---
///
/// 断点标识符类型。
pub type BreakpointId = u64;

/// Condition for conditional breakpoints.
///
/// ---
///
/// 条件断点的条件。
#[derive(Clone, Debug)]
pub enum BreakCondition {
    /// No condition - always break (无条件 - 总是中断)
    None,
    /// Break when register equals value (寄存器等于值时中断)
    RegisterEqual { reg: u8, value: u128 },
    /// Break when register not equals value (寄存器不等于值时中断)
    RegisterNotEqual { reg: u8, value: u128 },
    /// Break when memory at address equals value (内存地址的值等于指定值时中断)
    MemoryEqual { addr: memory::Address128, value: u64, size: u8 },
    /// Break when cycle count reaches value (周期计数达到值时中断)
    CycleCount(u64),
    /// Break when PC is in range (PC 在范围内时中断)
    PcInRange { start: memory::Address128, end: memory::Address128 },
}

/// Breakpoint type.
///
/// ---
///
/// 断点类型。
#[derive(Clone, Debug, PartialEq)]
pub enum BreakpointType {
    /// Software breakpoint at address (地址处的软件断点)
    Address(memory::Address128),
    /// Temporary breakpoint (removed after hit) (临时断点，触发后删除)
    Temporary(memory::Address128),
    /// Break on function entry (函数入口断点)
    Function(String),
}

/// A breakpoint definition.
///
/// ---
///
/// 断点定义。
#[derive(Clone, Debug)]
pub struct Breakpoint {
    /// Unique breakpoint identifier (唯一断点标识符)
    pub id: BreakpointId,
    /// Breakpoint type (断点类型)
    pub typ: BreakpointType,
    /// Condition for triggering (触发条件)
    pub condition: BreakCondition,
    /// Is breakpoint enabled (断点是否启用)
    pub enabled: bool,
    /// Hit count (命中次数)
    pub hit_count: u64,
    /// Ignore count - break after N hits (忽略次数 - N 次命中后中断)
    pub ignore_count: u64,
    /// Associated command to execute when hit (命中时执行的关联命令)
    pub commands: Vec<String>,
}

impl Breakpoint {
    /// Creates a new breakpoint at an address.
    ///
    /// ---
    ///
    /// 在地址处创建新断点。
    pub fn new_address(id: BreakpointId, address: memory::Address128) -> Self {
        Self {
            id,
            typ: BreakpointType::Address(address),
            condition: BreakCondition::None,
            enabled: true,
            hit_count: 0,
            ignore_count: 0,
            commands: Vec::new(),
        }
    }

    /// Creates a new temporary breakpoint.
    ///
    /// ---
    ///
    /// 创建新临时断点。
    pub fn new_temporary(id: BreakpointId, address: memory::Address128) -> Self {
        Self {
            id,
            typ: BreakpointType::Temporary(address),
            condition: BreakCondition::None,
            enabled: true,
            hit_count: 0,
            ignore_count: 0,
            commands: Vec::new(),
        }
    }

    /// Creates a new function breakpoint.
    ///
    /// ---
    ///
    /// 创建新函数断点。
    pub fn new_function(id: BreakpointId, name: String) -> Self {
        Self {
            id,
            typ: BreakpointType::Function(name),
            condition: BreakCondition::None,
            enabled: true,
            hit_count: 0,
            ignore_count: 0,
            commands: Vec::new(),
        }
    }

    /// Sets a condition on the breakpoint.
    ///
    /// ---
    ///
    /// 设置断点条件。
    pub fn with_condition(mut self, condition: BreakCondition) -> Self {
        self.condition = condition;
        self
    }

    /// Sets the ignore count.
    ///
    /// ---
    ///
    /// 设置忽略次数。
    pub fn with_ignore_count(mut self, count: u64) -> Self {
        self.ignore_count = count;
        self
    }

    /// Returns the address of this breakpoint, if applicable.
    ///
    /// ---
    ///
    /// 返回此断点的地址（如适用）。
    pub fn address(&self) -> Option<memory::Address128> {
        match &self.typ {
            BreakpointType::Address(addr) => Some(*addr),
            BreakpointType::Temporary(addr) => Some(*addr),
            BreakpointType::Function(_) => None,
        }
    }
}

/// Breakpoint manager.
///
/// Handles breakpoint creation, deletion, and querying.
///
/// ---
///
/// 断点管理器。
///
/// 处理断点的创建、删除和查询。
pub struct BreakpointManager {
    breakpoints: HashMap<BreakpointId, Breakpoint>,
    address_map: HashMap<memory::Address128, BreakpointId>,
    next_id: BreakpointId,
}

impl BreakpointManager {
    /// Creates a new breakpoint manager.
    ///
    /// ---
    ///
    /// 创建新断点管理器。
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            address_map: HashMap::new(),
            next_id: 1,
        }
    }

    /// Adds a breakpoint.
    ///
    /// Returns the breakpoint ID.
    ///
    /// ---
    ///
    /// 添加断点。
    ///
    /// 返回断点 ID。
    pub fn add(&mut self, breakpoint: Breakpoint) -> BreakpointId {
        let id = breakpoint.id;
        if let Some(addr) = breakpoint.address() {
            self.address_map.insert(addr, id);
        }
        self.breakpoints.insert(id, breakpoint);
        if id >= self.next_id {
            self.next_id = id + 1;
        }
        id
    }

    /// Creates and adds an address breakpoint.
    ///
    /// ---
    ///
    /// 创建并添加地址断点。
    pub fn add_address(&mut self, address: memory::Address128) -> BreakpointId {
        let id = self.next_id;
        self.next_id += 1;
        let bp = Breakpoint::new_address(id, address);
        self.add(bp)
    }

    /// Creates and adds a conditional breakpoint.
    ///
    /// ---
    ///
    /// 创建并添加条件断点。
    pub fn add_conditional(&mut self, address: memory::Address128, condition: BreakCondition) -> BreakpointId {
        let id = self.next_id;
        self.next_id += 1;
        let bp = Breakpoint::new_address(id, address).with_condition(condition);
        self.add(bp)
    }

    /// Creates and adds a temporary breakpoint.
    ///
    /// ---
    ///
    /// 创建并添加临时断点。
    pub fn add_temporary(&mut self, address: memory::Address128) -> BreakpointId {
        let id = self.next_id;
        self.next_id += 1;
        let bp = Breakpoint::new_temporary(id, address);
        self.add(bp)
    }

    /// Removes a breakpoint by ID.
    ///
    /// Returns the removed breakpoint, if any.
    ///
    /// ---
    ///
    /// 按 ID 删除断点。
    ///
    /// 返回被删除的断点（如有）。
    pub fn remove(&mut self, id: BreakpointId) -> Option<Breakpoint> {
        if let Some(bp) = self.breakpoints.remove(&id) {
            if let Some(addr) = bp.address() {
                self.address_map.remove(&addr);
            }
            Some(bp)
        } else {
            None
        }
    }

    /// Removes a breakpoint at an address.
    ///
    /// ---
    ///
    /// 删除地址处的断点。
    pub fn remove_at(&mut self, address: memory::Address128) -> Option<Breakpoint> {
        if let Some(id) = self.address_map.remove(&address) {
            self.breakpoints.remove(&id)
        } else {
            None
        }
    }

    /// Enables a breakpoint.
    ///
    /// ---
    ///
    /// 启用断点。
    pub fn enable(&mut self, id: BreakpointId) -> bool {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disables a breakpoint.
    ///
    /// ---
    ///
    /// 禁用断点。
    pub fn disable(&mut self, id: BreakpointId) -> bool {
        if let Some(bp) = self.breakpoints.get_mut(&id) {
            bp.enabled = false;
            true
        } else {
            false
        }
    }

    /// Gets a breakpoint by ID.
    ///
    /// ---
    ///
    /// 按 ID 获取断点。
    pub fn get(&self, id: BreakpointId) -> Option<&Breakpoint> {
        self.breakpoints.get(&id)
    }

    /// Gets a mutable breakpoint by ID.
    ///
    /// ---
    ///
    /// 按 ID 获取可变断点。
    pub fn get_mut(&mut self, id: &BreakpointId) -> Option<&mut Breakpoint> {
        self.breakpoints.get_mut(id)
    }

    /// Gets a breakpoint at an address.
    ///
    /// ---
    ///
    /// 获取地址处的断点。
    pub fn get_at(&self, address: memory::Address128) -> Option<&Breakpoint> {
        self.address_map.get(&address).and_then(|id| self.breakpoints.get(id))
    }

    /// Gets a mutable breakpoint at an address.
    ///
    /// ---
    ///
    /// 获取地址处的可变断点。
    pub fn get_at_mut(&mut self, address: memory::Address128) -> Option<&mut Breakpoint> {
        if let Some(id) = self.address_map.get(&address).copied() {
            self.breakpoints.get_mut(&id)
        } else {
            None
        }
    }

    /// Lists all breakpoints.
    ///
    /// ---
    ///
    /// 列出所有断点。
    pub fn list(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// Clears all breakpoints.
    ///
    /// ---
    ///
    /// 清除所有断点。
    pub fn clear(&mut self) {
        self.breakpoints.clear();
        self.address_map.clear();
    }

    /// Checks if there's a breakpoint at an address.
    ///
    /// ---
    ///
    /// 检查地址处是否有断点。
    pub fn has_breakpoint_at(&self, address: memory::Address128) -> bool {
        self.address_map.contains_key(&address)
    }
}

impl Default for BreakpointManager {
    fn default() -> Self {
        Self::new()
    }
}

// ========================================
// Watchpoint System / 观察点系统
// ========================================

/// Watchpoint identifier type.
///
/// ---
///
/// 观察点标识符类型。
pub type WatchpointId = u64;

/// Watchpoint type.
///
/// ---
///
/// 观察点类型。
#[derive(Clone, Debug, PartialEq)]
pub enum WatchpointType {
    /// Watch for reads (监视读取)
    Read,
    /// Watch for writes (监视写入)
    Write,
    /// Watch for both reads and writes (监视读取和写入)
    Access,
}

/// A watchpoint definition.
///
/// ---
///
/// 观察点定义。
#[derive(Clone, Debug)]
pub struct Watchpoint {
    /// Unique watchpoint identifier (唯一观察点标识符)
    pub id: WatchpointId,
    /// Memory address to watch (要监视的内存地址)
    pub address: memory::Address128,
    /// Size in bytes (字节大小)
    pub size: usize,
    /// Watchpoint type (观察点类型)
    pub typ: WatchpointType,
    /// Is watchpoint enabled (观察点是否启用)
    pub enabled: bool,
    /// Trigger count (触发次数)
    pub trigger_count: u64,
    /// Old value (for change detection) (旧值，用于变化检测)
    pub old_value: Option<Vec<u8>>,
}

impl Watchpoint {
    /// Creates a new watchpoint.
    ///
    /// ---
    ///
    /// 创建新观察点。
    pub fn new(id: WatchpointId, address: memory::Address128, size: usize, typ: WatchpointType) -> Self {
        Self {
            id,
            address,
            size,
            typ,
            enabled: true,
            trigger_count: 0,
            old_value: None,
        }
    }

    /// Checks if this watchpoint covers the given address range and access type.
    ///
    /// ---
    ///
    /// 检查此观察点是否覆盖给定的地址范围和访问类型。
    pub fn matches(&self, addr: memory::Address128, size: usize, is_write: bool) -> bool {
        if !self.enabled {
            return false;
        }

        let typ_match = match self.typ {
            WatchpointType::Read => !is_write,
            WatchpointType::Write => is_write,
            WatchpointType::Access => true,
        };

        if !typ_match {
            return false;
        }

        // Check address range overlap
        let self_end = self.address + self.size as u128;
        let access_end = addr + size as u128;
        addr < self_end && self.address < access_end
    }
}

/// Register watchpoint.
///
/// ---
///
/// 寄存器观察点。
#[derive(Clone, Debug)]
pub struct RegisterWatchpoint {
    /// Unique watchpoint identifier (唯一观察点标识符)
    pub id: WatchpointId,
    /// Register index (寄存器索引)
    pub reg: u8,
    /// Is it a floating-point register (是否为浮点寄存器)
    pub is_fp: bool,
    /// Watch for changes (监视变化)
    pub enabled: bool,
    /// Old value (旧值)
    pub old_value: u128,
}

impl RegisterWatchpoint {
    /// Creates a new register watchpoint.
    ///
    /// ---
    ///
    /// 创建新寄存器观察点。
    pub fn new(id: WatchpointId, reg: u8, is_fp: bool) -> Self {
        Self {
            id,
            reg,
            is_fp,
            enabled: true,
            old_value: 0,
        }
    }
}

/// Watchpoint manager.
///
/// ---
///
/// 观察点管理器。
pub struct WatchpointManager {
    watchpoints: HashMap<WatchpointId, Watchpoint>,
    register_watchpoints: HashMap<WatchpointId, RegisterWatchpoint>,
    next_id: WatchpointId,
}

impl WatchpointManager {
    /// Creates a new watchpoint manager.
    ///
    /// ---
    ///
    /// 创建新观察点管理器。
    pub fn new() -> Self {
        Self {
            watchpoints: HashMap::new(),
            register_watchpoints: HashMap::new(),
            next_id: 1,
        }
    }

    /// Adds a memory watchpoint.
    ///
    /// ---
    ///
    /// 添加内存观察点。
    pub fn add(&mut self, watchpoint: Watchpoint) -> WatchpointId {
        let id = watchpoint.id;
        self.watchpoints.insert(id, watchpoint);
        if id >= self.next_id {
            self.next_id = id + 1;
        }
        id
    }

    /// Creates and adds a memory watchpoint.
    ///
    /// ---
    ///
    /// 创建并添加内存观察点。
    pub fn add_memory(&mut self, address: memory::Address128, size: usize, typ: WatchpointType) -> WatchpointId {
        let id = self.next_id;
        self.next_id += 1;
        let wp = Watchpoint::new(id, address, size, typ);
        self.add(wp)
    }

    /// Adds a register watchpoint.
    ///
    /// ---
    ///
    /// 添加寄存器观察点。
    pub fn add_register(&mut self, reg: u8, is_fp: bool) -> WatchpointId {
        let id = self.next_id;
        self.next_id += 1;
        let wp = RegisterWatchpoint::new(id, reg, is_fp);
        self.register_watchpoints.insert(id, wp);
        id
    }

    /// Removes a watchpoint by ID.
    ///
    /// ---
    ///
    /// 按 ID 删除观察点。
    pub fn remove(&mut self, id: WatchpointId) -> bool {
        self.watchpoints.remove(&id).is_some() || self.register_watchpoints.remove(&id).is_some()
    }

    /// Lists all memory watchpoints.
    ///
    /// ---
    ///
    /// 列出所有内存观察点。
    pub fn list_memory(&self) -> Vec<&Watchpoint> {
        self.watchpoints.values().collect()
    }

    /// Lists all register watchpoints.
    ///
    /// ---
    ///
    /// 列出所有寄存器观察点。
    pub fn list_registers(&self) -> Vec<&RegisterWatchpoint> {
        self.register_watchpoints.values().collect()
    }

    /// Checks for triggered watchpoints on memory access.
    ///
    /// ---
    ///
    /// 检查内存访问时触发的观察点。
    pub fn check_memory(&self, addr: memory::Address128, size: usize, is_write: bool) -> Vec<&Watchpoint> {
        self.watchpoints.values()
            .filter(|wp| wp.matches(addr, size, is_write))
            .collect()
    }

    /// Updates watchpoint with new value.
    ///
    /// ---
    ///
    /// 用新值更新观察点。
    pub fn update_value(&mut self, id: WatchpointId, value: Vec<u8>) {
        if let Some(wp) = self.watchpoints.get_mut(&id) {
            wp.old_value = Some(value);
        }
    }

    /// Clears all watchpoints.
    ///
    /// ---
    ///
    /// 清除所有观察点。
    pub fn clear(&mut self) {
        self.watchpoints.clear();
        self.register_watchpoints.clear();
    }
}

impl Default for WatchpointManager {
    fn default() -> Self {
        Self::new()
    }
}

// ========================================
// Execution History / 执行历史
// ========================================

/// A single execution history entry.
///
/// ---
///
/// 单个执行历史条目。
#[derive(Clone, Debug)]
pub struct HistoryEntry {
    /// Cycle number (周期编号)
    pub cycle: u64,
    /// Program counter (程序计数器)
    pub pc: memory::Address128,
    /// Raw instruction bytes (原始指令字节)
    pub instruction: u32,
    /// Disassembled instruction (反汇编指令)
    pub disassembly: String,
    /// Register state before execution (执行前的寄存器状态)
    pub registers_before: [u128; 32],
    /// PC before execution (执行前的 PC)
    pub pc_before: memory::Address128,
    /// Modified register (if any) (修改的寄存器，如有)
    pub modified_reg: Option<u8>,
    /// New value of modified register (修改寄存器的新值)
    pub new_value: Option<u128>,
    /// Memory writes (内存写入)
    pub memory_writes: Vec<(memory::Address128, Vec<u8>)>,
}

impl HistoryEntry {
    /// Creates a new history entry.
    ///
    /// ---
    ///
    /// 创建新历史条目。
    pub fn new(
        cycle: u64,
        pc: memory::Address128,
        instruction: u32,
        registers_before: [u128; 32],
        pc_before: memory::Address128,
    ) -> Self {
        let disassembly = assembler::disassemble_instruction(instruction);
        Self {
            cycle,
            pc,
            instruction,
            disassembly,
            registers_before,
            pc_before,
            modified_reg: None,
            new_value: None,
            memory_writes: Vec::new(),
        }
    }

    /// Records a register modification.
    ///
    /// ---
    ///
    /// 记录寄存器修改。
    pub fn record_register_change(&mut self, reg: u8, new_value: u128) {
        self.modified_reg = Some(reg);
        self.new_value = Some(new_value);
    }

    /// Records a memory write.
    ///
    /// ---
    ///
    /// 记录内存写入。
    pub fn record_memory_write(&mut self, addr: memory::Address128, data: Vec<u8>) {
        self.memory_writes.push((addr, data));
    }
}

/// Execution history manager.
///
/// ---
///
/// 执行历史管理器。
pub struct ExecutionHistory {
    entries: Vec<HistoryEntry>,
    max_entries: usize,
    current_index: usize,
}

impl ExecutionHistory {
    /// Creates a new execution history with a maximum size.
    ///
    /// ---
    ///
    /// 创建具有最大大小的新执行历史。
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            max_entries,
            current_index: 0,
        }
    }

    /// Adds a new history entry.
    ///
    /// ---
    ///
    /// 添加新历史条目。
    pub fn add(&mut self, entry: HistoryEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        } else {
            self.current_index = self.entries.len();
        }
        self.entries.push(entry);
    }

    /// Gets the current entry.
    ///
    /// ---
    ///
    /// 获取当前条目。
    pub fn current(&self) -> Option<&HistoryEntry> {
        self.entries.get(self.current_index)
    }

    /// Gets an entry by index.
    ///
    /// ---
    ///
    /// 按索引获取条目。
    pub fn get(&self, index: usize) -> Option<&HistoryEntry> {
        self.entries.get(index)
    }

    /// Gets the last N entries.
    ///
    /// ---
    ///
    /// 获取最后 N 个条目。
    pub fn last_n(&self, n: usize) -> Vec<&HistoryEntry> {
        let start = if self.entries.len() > n {
            self.entries.len() - n
        } else {
            0
        };
        self.entries[start..].iter().collect()
    }

    /// Clears all history.
    ///
    /// ---
    ///
    /// 清除所有历史。
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_index = 0;
    }

    /// Returns the number of entries.
    ///
    /// ---
    ///
    /// 返回条目数量。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if history is empty.
    ///
    /// ---
    ///
    /// 如果历史为空则返回 true。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ========================================
// Debugger State / 调试器状态
// ========================================

/// Debugger stop reason.
///
/// ---
///
/// 调试器停止原因。
#[derive(Clone, Debug)]
pub enum StopReason {
    /// Breakpoint hit (断点命中)
    Breakpoint(BreakpointId),
    /// Watchpoint triggered (观察点触发)
    Watchpoint(WatchpointId),
    /// Step completed (单步完成)
    Step,
    /// Next completed (next 完成)
    Next,
    /// Continue until address (继续到地址)
    Until(memory::Address128),
    /// Exception occurred (发生异常)
    Exception(String),
    /// User interrupted (用户中断)
    Interrupted,
    /// Program exited (程序退出)
    Exit,
    /// Halted (停止)
    Halted,
}

/// Debugger action to take.
///
/// ---
///
/// 要采取的调试器动作。
#[derive(Clone, Debug, PartialEq)]
pub enum DebuggerAction {
    /// Continue execution (继续执行)
    Continue,
    /// Single step (单步执行)
    Step,
    /// Step over (next) (步过)
    Next,
    /// Step out (步出)
    Finish,
    /// Stop execution (停止执行)
    Stop,
    /// Restart execution (重新执行)
    Restart,
}

// ========================================
// Interactive Debugger / 交互式调试器
// ========================================

/// Interactive debugger command.
///
/// ---
///
/// 交互式调试器命令。
#[derive(Clone, Debug)]
pub enum DebugCommand {
    /// Continue execution (继续执行)
    Continue,
    /// Single step (单步执行)
    Step,
    /// Step over (步过 - 不进入函数)
    Next,
    /// Step out of current function (步出当前函数)
    Finish,
    /// Run until address (运行到地址)
    Until(memory::Address128),
    /// Set breakpoint at address (在地址设置断点)
    Break(memory::Address128),
    /// Set breakpoint with condition (设置带条件的断点)
    BreakConditional(memory::Address128, BreakCondition),
    /// Set temporary breakpoint (设置临时断点)
    TBreak(memory::Address128),
    /// Delete breakpoint (删除断点)
    Delete(Option<BreakpointId>),
    /// Enable breakpoint (启用断点)
    Enable(BreakpointId),
    /// Disable breakpoint (禁用断点)
    Disable(BreakpointId),
    /// Set ignore count for breakpoint (设置断点忽略次数)
    Ignore(BreakpointId, u64),
    /// List breakpoints (列出断点)
    ListBreakpoints,
    /// Set watchpoint (设置观察点)
    Watch(memory::Address128, usize, WatchpointType),
    /// Watch register (监视寄存器)
    WatchReg(u8, bool),
    /// Delete watchpoint (删除观察点)
    DeleteWatch(WatchpointId),
    /// List watchpoints (列出观察点)
    ListWatchpoints,
    /// Print register value (打印寄存器值)
    PrintRegister(u8),
    /// Print all registers (打印所有寄存器)
    PrintAllRegisters,
    /// Print memory (打印内存)
    PrintMemory(memory::Address128, usize),
    /// Set register value (设置寄存器值)
    SetRegister(u8, u128),
    /// Set memory value (设置内存值)
    SetMemory(memory::Address128, Vec<u8>),
    /// Disassemble at address (在地址反汇编)
    Disassemble(memory::Address128, usize),
    /// Show execution history (显示执行历史)
    History(usize),
    /// Show backtrace (显示调用栈)
    Backtrace,
    /// Show current position (显示当前位置)
    Where,
    /// Reset VM (重置虚拟机)
    Reset,
    /// Show help (显示帮助)
    Help,
    /// Quit debugger (退出调试器)
    Quit,
    /// Execute raw command (执行原始命令)
    Raw(String),
}

/// Command parser for interactive debugger.
///
/// ---
///
/// 交互式调试器的命令解析器。
pub struct CommandParser;

impl CommandParser {
    /// Parses a command string.
    ///
    /// ---
    ///
    /// 解析命令字符串。
    pub fn parse(input: &str) -> Option<DebugCommand> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let cmd = parts[0].to_lowercase();
        match cmd.as_str() {
            "c" | "continue" | "cont" => Some(DebugCommand::Continue),
            "s" | "step" => Some(DebugCommand::Step),
            "n" | "next" => Some(DebugCommand::Next),
            "finish" | "fin" => Some(DebugCommand::Finish),
            "until" | "u" => {
                if parts.len() > 1 {
                    Self::parse_address(parts[1]).map(|addr| DebugCommand::Until(addr))
                } else {
                    None
                }
            }
            "b" | "break" | "bp" => {
                if parts.len() > 1 {
                    Self::parse_address(parts[1]).map(|addr| DebugCommand::Break(addr))
                } else {
                    None
                }
            }
            "tb" | "tbreak" => {
                if parts.len() > 1 {
                    Self::parse_address(parts[1]).map(|addr| DebugCommand::TBreak(addr))
                } else {
                    None
                }
            }
            "d" | "delete" | "del" => {
                if parts.len() > 1 {
                    Self::parse_u64(parts[1]).map(|id| DebugCommand::Delete(Some(id)))
                } else {
                    Some(DebugCommand::Delete(None))
                }
            }
            "enable" | "en" => {
                if parts.len() > 1 {
                    Self::parse_u64(parts[1]).map(|id| DebugCommand::Enable(id))
                } else {
                    None
                }
            }
            "disable" | "dis" => {
                if parts.len() > 1 {
                    Self::parse_u64(parts[1]).map(|id| DebugCommand::Disable(id))
                } else {
                    None
                }
            }
            "ignore" => {
                if parts.len() > 2 {
                    let id = Self::parse_u64(parts[1]);
                    let count = Self::parse_u64(parts[2]);
                    match (id, count) {
                        (Some(id), Some(count)) => Some(DebugCommand::Ignore(id, count)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "info" | "i" => {
                if parts.len() > 1 {
                    match parts[1].to_lowercase().as_str() {
                        "b" | "break" | "breakpoints" => Some(DebugCommand::ListBreakpoints),
                        "w" | "watch" | "watchpoints" => Some(DebugCommand::ListWatchpoints),
                        "r" | "reg" | "registers" => Some(DebugCommand::PrintAllRegisters),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            "watch" | "wa" => {
                if parts.len() > 1 {
                    let addr = Self::parse_address(parts[1]);
                    let size = if parts.len() > 2 {
                        Self::parse_usize(parts[2]).unwrap_or(8)
                    } else {
                        8
                    };
                    let typ = if parts.len() > 3 {
                        match parts[3].to_lowercase().as_str() {
                            "r" | "read" => WatchpointType::Read,
                            "w" | "write" => WatchpointType::Write,
                            _ => WatchpointType::Access,
                        }
                    } else {
                        WatchpointType::Write
                    };
                    addr.map(|a| DebugCommand::Watch(a, size, typ))
                } else {
                    None
                }
            }
            "watchreg" | "wr" => {
                if parts.len() > 1 {
                    let reg = Self::parse_u8(parts[1]);
                    let is_fp = parts.len() > 2 && (parts[2] == "fp" || parts[2] == "f");
                    reg.map(|r| DebugCommand::WatchReg(r, is_fp))
                } else {
                    None
                }
            }
            "dwatch" | "deletewatch" => {
                if parts.len() > 1 {
                    Self::parse_u64(parts[1]).map(|id| DebugCommand::DeleteWatch(id))
                } else {
                    None
                }
            }
            "p" | "print" => {
                if parts.len() > 1 {
                    // Check if it's a register
                    if parts[1].starts_with('$') || parts[1].starts_with('x') || parts[1].starts_with('r') {
                        let reg_str = parts[1].trim_start_matches('$').trim_start_matches('x').trim_start_matches('r');
                        Self::parse_u8(reg_str).map(|r| DebugCommand::PrintRegister(r))
                    } else {
                        // Treat as memory address
                        let addr = Self::parse_address(parts[1]);
                        let size = if parts.len() > 2 {
                            Self::parse_usize(parts[2]).unwrap_or(16)
                        } else {
                            16
                        };
                        addr.map(|a| DebugCommand::PrintMemory(a, size))
                    }
                } else {
                    None
                }
            }
            "reg" | "registers" => Some(DebugCommand::PrintAllRegisters),
            "set" => {
                if parts.len() > 2 {
                    // set reg value or set addr value
                    let target = parts[1];
                    let value_str = parts[2];
                    if target.starts_with('$') || target.starts_with('x') || target.starts_with('r') {
                        let reg_str = target.trim_start_matches('$').trim_start_matches('x').trim_start_matches('r');
                        let reg = Self::parse_u8(reg_str);
                        let value = Self::parse_u128(value_str);
                        match (reg, value) {
                            (Some(r), Some(v)) => Some(DebugCommand::SetRegister(r, v)),
                            _ => None,
                        }
                    } else {
                        let addr = Self::parse_address(target);
                        // For memory, parse hex bytes
                        let bytes: Vec<u8> = value_str.as_bytes()
                            .chunks(2)
                            .filter_map(|chunk| {
                                let s = std::str::from_utf8(chunk).ok()?;
                                u8::from_str_radix(s, 16).ok()
                            })
                            .collect();
                        addr.map(|a| DebugCommand::SetMemory(a, bytes))
                    }
                } else {
                    None
                }
            }
            "x" | "examine" | "mem" => {
                if parts.len() > 1 {
                    let addr = Self::parse_address(parts[1]);
                    let size = if parts.len() > 2 {
                        Self::parse_usize(parts[2]).unwrap_or(16)
                    } else {
                        16
                    };
                    addr.map(|a| DebugCommand::PrintMemory(a, size))
                } else {
                    None
                }
            }
            "disasm" | "disassemble" => {
                if parts.len() > 1 {
                    let addr = Self::parse_address(parts[1]);
                    let count = if parts.len() > 2 {
                        Self::parse_usize(parts[2]).unwrap_or(10)
                    } else {
                        10
                    };
                    addr.map(|a| DebugCommand::Disassemble(a, count))
                } else {
                    None
                }
            }
            "history" | "hist" | "h" => {
                let count = if parts.len() > 1 {
                    Self::parse_usize(parts[1]).unwrap_or(10)
                } else {
                    10
                };
                Some(DebugCommand::History(count))
            }
            "bt" | "backtrace" => Some(DebugCommand::Backtrace),
            "where" | "w" => Some(DebugCommand::Where),
            "reset" => Some(DebugCommand::Reset),
            "help" | "?" => Some(DebugCommand::Help),
            "q" | "quit" | "exit" => Some(DebugCommand::Quit),
            _ => Some(DebugCommand::Raw(input.to_string())),
        }
    }

    fn parse_address(s: &str) -> Option<memory::Address128> {
        let s = s.trim();
        if s.starts_with("0x") || s.starts_with("0X") {
            u128::from_str_radix(&s[2..], 16).ok()
        } else {
            u128::from_str_radix(s, 0).ok()
        }
    }

    fn parse_u64(s: &str) -> Option<u64> {
        let s = s.trim();
        if s.starts_with("0x") || s.starts_with("0X") {
            u64::from_str_radix(&s[2..], 16).ok()
        } else {
            u64::from_str_radix(s, 0).ok()
        }
    }

    fn parse_u8(s: &str) -> Option<u8> {
        Self::parse_u64(s).and_then(|v| if v <= 31 { Some(v as u8) } else { None })
    }

    fn parse_usize(s: &str) -> Option<usize> {
        Self::parse_u64(s).map(|v| v as usize)
    }

    fn parse_u128(s: &str) -> Option<u128> {
        let s = s.trim();
        if s.starts_with("0x") || s.starts_with("0X") {
            u128::from_str_radix(&s[2..], 16).ok()
        } else {
            u128::from_str_radix(s, 0).ok()
        }
    }
}

/// Interactive debugger interface.
///
/// ---
///
/// 交互式调试器界面。
pub struct InteractiveDebugger {
    pub breakpoints: BreakpointManager,
    pub watchpoints: WatchpointManager,
    pub history: ExecutionHistory,
    pub action: DebuggerAction,
    pub stop_reason: Option<StopReason>,
    /// Temporary breakpoint ID (for next/finish/until) (临时断点 ID)
    pub temp_breakpoint: Option<BreakpointId>,
    finish_return_addr: Option<memory::Address128>,
}

impl InteractiveDebugger {
    /// Creates a new interactive debugger.
    ///
    /// ---
    ///
    /// 创建新交互式调试器。
    pub fn new(history_size: usize) -> Self {
        Self {
            breakpoints: BreakpointManager::new(),
            watchpoints: WatchpointManager::new(),
            history: ExecutionHistory::new(history_size),
            action: DebuggerAction::Stop,
            stop_reason: Some(StopReason::Halted),
            temp_breakpoint: None,
            finish_return_addr: None,
        }
    }

    /// Prints help message.
    ///
    /// ---
    ///
    /// 打印帮助信息。
    pub fn print_help(&self) {
        println!("RISC-V 128-bit VM Debugger Commands:");
        println!();
        println!("Execution Control:");
        println!("  c, continue          Continue execution");
        println!("  s, step              Single step (enter function calls)");
        println!("  n, next              Step over (skip function calls)");
        println!("  finish               Step out of current function");
        println!("  u, until <addr>      Run until address");
        println!();
        println!("Breakpoints:");
        println!("  b, break <addr>      Set breakpoint at address");
        println!("  tb, tbreak <addr>    Set temporary breakpoint");
        println!("  d, delete [id]       Delete breakpoint (all if no id)");
        println!("  enable <id>          Enable breakpoint");
        println!("  disable <id>         Disable breakpoint");
        println!("  ignore <id> <n>      Ignore breakpoint for N hits");
        println!("  info b               List breakpoints");
        println!();
        println!("Watchpoints:");
        println!("  watch <addr> [size] [type]  Set memory watchpoint");
        println!("                         type: r(ead)/w(rite)/a(ccess)");
        println!("  watchreg <reg> [fp]  Watch register change");
        println!("  dwatch <id>          Delete watchpoint");
        println!("  info w               List watchpoints");
        println!();
        println!("Inspection:");
        println!("  p, print <reg|$n>    Print register value");
        println!("  x, mem <addr> [size] Examine memory");
        println!("  reg, registers       Print all registers");
        println!("  dis <addr> [count]   Disassemble instructions");
        println!("  history [n]          Show execution history (last N)");
        println!("  bt, backtrace        Show call stack");
        println!("  where                Show current position");
        println!();
        println!("Modification:");
        println!("  set <$n> <value>     Set register value");
        println!("  set <addr> <hex>     Set memory (hex bytes)");
        println!("  reset                Reset VM state");
        println!();
        println!("Other:");
        println!("  help, ?              Show this help");
        println!("  q, quit              Quit debugger");
    }

    /// Displays current position.
    ///
    /// ---
    ///
    /// 显示当前位置。
    pub fn show_position(&self, pc: memory::Address128, instruction: u32, cycle: u64) {
        let asm = assembler::disassemble_instruction(instruction);
        println!("Cycle {} | PC: 0x{:016x} | {}", cycle, pc, asm);
    }

    /// Displays register value.
    ///
    /// ---
    ///
    /// 显示寄存器值。
    pub fn show_register(&self, reg: u8, value: u128) {
        let abi_name = register::abi_name(reg);
        println!("x{} ({}): 0x{:032x} ({})", reg, abi_name, value, value as i128);
    }

    /// Displays all registers.
    ///
    /// ---
    ///
    /// 显示所有寄存器。
    pub fn show_all_registers(&self, regs: &[u128; 32], pc: memory::Address128) {
        println!("PC: 0x{:032x}", pc);
        println!("Registers:");
        for i in 0..32 {
            let abi = register::abi_name(i as u8);
            print!("  x{:02} ({:>4}): 0x{:032x}", i, abi, regs[i]);
            if (i + 1) % 2 == 0 {
                println!();
            } else {
                print!("  ");
            }
        }
    }

    /// Displays memory range.
    ///
    /// ---
    ///
    /// 显示内存范围。
    pub fn show_memory(&self, memory: &memory::Memory, start: memory::Address128, size: usize) {
        println!("Memory [0x{:016x} - 0x{:016x}]:", start, start + size as u128);
        for i in (0..size).step_by(16) {
            let addr = start + i as u128;
            print!("  0x{:016x}: ", addr);
            for j in 0..16 {
                if i + j < size {
                    let byte = memory.read_8(start + (i + j) as u128);
                    print!("{:02x}", byte);
                } else {
                    print!("  ");
                }
                if (j + 1) % 4 == 0 {
                    print!(" ");
                }
            }
            println!();
        }
    }

    /// Displays disassembly.
    ///
    /// ---
    ///
    /// 显示反汇编。
    pub fn show_disassembly(&self, memory: &memory::Memory, start: memory::Address128, count: usize) {
        println!("Disassembly from 0x{:016x}:", start);
        let mut addr = start;
        for _ in 0..count {
            let instr = memory.read_32(addr);
            let asm = assembler::disassemble_instruction(instr);
            println!("  0x{:016x}: {:08x}  {}", addr, instr, asm);
            addr += 4;
        }
    }

    /// Displays execution history.
    ///
    /// ---
    ///
    /// 显示执行历史。
    pub fn show_history(&self, count: usize) {
        let entries = self.history.last_n(count);
        if entries.is_empty() {
            println!("No execution history.");
            return;
        }

        println!("Execution history (last {}):", entries.len());
        for entry in entries {
            print!("  [{:6}] 0x{:016x}: {:08x} {}", 
                   entry.cycle, entry.pc, entry.instruction, entry.disassembly);
            if let Some(reg) = entry.modified_reg {
                if let Some(val) = entry.new_value {
                    print!(" -> x{} = 0x{:016x}", reg, val);
                }
            }
            println!();
        }
    }

    /// Displays breakpoints.
    ///
    /// ---
    ///
    /// 显示断点列表。
    pub fn show_breakpoints(&self) {
        let breakpoints = self.breakpoints.list();
        if breakpoints.is_empty() {
            println!("No breakpoints.");
            return;
        }

        println!("Breakpoints:");
        for bp in breakpoints {
            let status = if bp.enabled { "enabled" } else { "disabled" };
            let addr_str = match &bp.typ {
                BreakpointType::Address(addr) => format!("0x{:016x}", addr),
                BreakpointType::Temporary(addr) => format!("0x{:016x} (temp)", addr),
                BreakpointType::Function(name) => name.clone(),
            };
            println!("  {}: {} [{}] hits: {}", bp.id, addr_str, status, bp.hit_count);
        }
    }

    /// Displays watchpoints.
    ///
    /// ---
    ///
    /// 显示观察点列表。
    pub fn show_watchpoints(&self) {
        let mem_wps = self.watchpoints.list_memory();
        let reg_wps = self.watchpoints.list_registers();

        if mem_wps.is_empty() && reg_wps.is_empty() {
            println!("No watchpoints.");
            return;
        }

        if !mem_wps.is_empty() {
            println!("Memory watchpoints:");
            for wp in mem_wps {
                let typ_str = match wp.typ {
                    WatchpointType::Read => "read",
                    WatchpointType::Write => "write",
                    WatchpointType::Access => "access",
                };
                println!("  {}: 0x{:016x} ({} bytes, {}) [{}]", 
                         wp.id, wp.address, wp.size, typ_str,
                         if wp.enabled { "enabled" } else { "disabled" });
            }
        }

        if !reg_wps.is_empty() {
            println!("Register watchpoints:");
            for wp in reg_wps {
                let reg_type = if wp.is_fp { "fp" } else { "gp" };
                println!("  {}: {}r{} [{}]", 
                         wp.id, reg_type, wp.reg,
                         if wp.enabled { "enabled" } else { "disabled" });
            }
        }
    }
}

impl Default for InteractiveDebugger {
    fn default() -> Self {
        Self::new(1000)
    }
}
