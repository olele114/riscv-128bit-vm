//! Memory Module
//!
//! Implements the memory system for the RISC-V virtual machine.
//! Supports region-based memory with configurable permissions.
//!
//! # Features
//!
//! - Region-based memory management
//! - Permission control (Read, Write, Execute)
//! - Support for 8/16/32/64/128-bit access
//! - Little-endian byte ordering
//! - Atomic memory operations (A extension)
//! - Load-Reserved/Store-Conditional (LR/SC)
//!
//! ---
//!
//! 内存模块
//!
//! 实现 RISC-V 虚拟机的内存系统。
//! 支持基于区域的内存和可配置权限。
//!
//! # 特性
//!
//! - 基于区域的内存管理
//! - 权限控制（读、写、执行）
//! - 支持 8/16/32/64/128 位访问
//! - 小端字节序
//! - 原子内存操作（A 扩展）
//! - 加载保留/存储条件（LR/SC）

#![allow(dead_code)]

use bitflags::bitflags;

/// 128-bit address type
///
/// ---
///
/// 128 位地址类型
pub(crate) type Address128 = u128;
/// 128-bit word type
///
/// ---
///
/// 128 位字类型
pub(crate) type Word128 = u128;
type Word64 = u64;
type Word32 = u32;
type Word16 = u16;
type Word8 = u8;

bitflags! {
    /// Memory access permissions.
    ///
    /// Controls what operations are allowed on a memory region.
    ///
    /// ---
    ///
    /// 内存访问权限。
    ///
    /// 控制内存区域允许的操作。
    #[derive(Clone, Copy, Debug)]
    pub struct MemoryPermissions : u8 {
        /// No permissions (无权限)
        const None = 0;
        /// Read permission (读权限)
        const Read = 1;
        /// Write permission (写权限)
        const Write = 2;
        /// Execute permission (执行权限)
        const Execute = 4;

        /// Read + Write (读 + 写)
        const ReadWrite = Self::Read.bits() | Self::Write.bits();
        /// Read + Execute (读 + 执行)
        const ReadExecute = Self::Read.bits() | Self::Execute.bits();
        /// All permissions (所有权限)
        const All = Self::ReadWrite.bits() | Self::Execute.bits();
    }
}

/// Reservation set entry for LR/SC instructions.
///
/// Each entry represents a reservation made by a LR instruction.
/// The reservation is invalidated if another hart writes to the
/// same address or address range.
///
/// ---
///
/// LR/SC 指令的保留集条目。
///
/// 每个条目代表由 LR 指令创建的保留。
/// 如果另一个硬件线程写入同一地址或地址范围，保留将被无效化。
#[derive(Clone, Debug)]
pub struct ReservationEntry {
    /// Address of the reservation (保留地址)
    pub address: Address128,
    /// Size of the reservation in bytes (保留大小，以字节为单位)
    pub size: u8,
    /// Whether the reservation is valid (保留是否有效)
    pub valid: bool,
}

/// Reservation set for LR/SC synchronization.
///
/// Implements the reservation set mechanism for Load-Reserved and
/// Store-Conditional instructions as defined in the RISC-V A extension.
///
/// In a single-threaded VM, reservations are always valid until
/// explicitly invalidated by a store or cleared.
///
/// ---
///
/// LR/SC 同步的保留集。
///
/// 实现 RISC-V A 扩展定义的加载保留和存储条件指令的保留集机制。
///
/// 在单线程虚拟机中，保留始终有效，直到被存储显式无效化或清除。
#[derive(Clone, Debug)]
pub struct ReservationSet {
    /// The current reservation (当前的保留)
    reservation: Option<ReservationEntry>,
}

impl ReservationSet {
    /// Creates a new empty reservation set.
    ///
    /// ---
    ///
    /// 创建新的空保留集。
    pub fn new() -> Self {
        Self { reservation: None }
    }

    /// Creates a reservation for the given address and size.
    ///
    /// Called by LR instructions.
    ///
    /// ---
    ///
    /// 为给定地址和大小创建保留。
    ///
    /// 由 LR 指令调用。
    pub fn set_reservation(&mut self, address: Address128, size: u8) {
        self.reservation = Some(ReservationEntry {
            address,
            size,
            valid: true,
        });
    }

    /// Checks if there is a valid reservation for the given address.
    ///
    /// Called by SC instructions. Returns true if the reservation
    /// matches and is valid.
    ///
    /// ---
    ///
    /// 检查给定地址是否有有效保留。
    ///
    /// 由 SC 指令调用。如果保留匹配且有效则返回 true。
    pub fn check_reservation(&self, address: Address128, size: u8) -> bool {
        if let Some(ref res) = self.reservation {
            // Check if address overlaps with reservation
            let res_end = res.address + res.size as Address128;
            let check_end = address + size as Address128;
            res.valid && res.address < check_end && address < res_end
        } else {
            false
        }
    }

    /// Invalidates the reservation.
    ///
    /// Called after a successful SC or when the reservation
    /// needs to be cleared.
    ///
    /// ---
    ///
    /// 无效化保留。
    ///
    /// 在成功的 SC 之后或需要清除保留时调用。
    pub fn clear_reservation(&mut self) {
        self.reservation = None;
    }

    /// Invalidates any reservation that overlaps with the given address range.
    ///
    /// Called when any store operation occurs (to maintain sequential
    /// consistency in single-threaded mode).
    ///
    /// ---
    ///
    /// 无效化与给定地址范围重叠的任何保留。
    ///
    /// 当发生任何存储操作时调用（以在单线程模式下保持顺序一致性）。
    pub fn invalidate_if_overlaps(&mut self, address: Address128, size: u8) {
        if let Some(ref res) = self.reservation {
            let res_end = res.address + res.size as Address128;
            let store_end = address + size as Address128;
            if res.address < store_end && address < res_end {
                self.reservation = None;
            }
        }
    }

    /// Returns whether there is an active reservation.
    ///
    /// ---
    ///
    /// 返回是否有活动保留。
    pub fn has_reservation(&self) -> bool {
        self.reservation.is_some() && self.reservation.as_ref().unwrap().valid
    }
}

impl Default for ReservationSet {
    fn default() -> Self {
        Self::new()
    }
}

/// A contiguous memory region with permissions.
///
/// ---
///
/// 具有权限的连续内存区域。
struct MemoryRegion {
    start: Address128,
    end: Address128,
    data: Vec<u8>,
    permissions: MemoryPermissions,
    name: String,
}

/// Memory system for the virtual machine.
///
/// Manages multiple memory regions with different permissions.
/// Default configuration creates a single read-write region.
///
/// Supports atomic memory operations (AMO) and Load-Reserved/Store-Conditional
/// for the RISC-V A extension.
///
/// ---
///
/// 虚拟机的内存系统。
///
/// 管理多个具有不同权限的内存区域。
/// 默认配置创建单个读写区域。
///
/// 支持 RISC-V A 扩展的原子内存操作（AMO）和加载保留/存储条件。
pub struct Memory {
    regions: Vec<MemoryRegion>,
    size: Address128,
    /// Reservation set for LR/SC instructions (LR/SC 指令的保留集)
    reservation_set: ReservationSet,
}

impl MemoryRegion {
    /// Creates a new memory region.
    ///
    /// ---
    ///
    /// 创建新内存区域。
    pub fn new(start: Address128, size: Address128,
               perms: MemoryPermissions, name: String) -> MemoryRegion {
        MemoryRegion {
            start,
            end: start + size,
            data: vec![0; size as usize],
            permissions: perms,
            name,
        }
    }
}

impl Memory {
    /// Default memory size: 16MB
    ///
    /// ---
    ///
    /// 默认内存大小：16MB
    pub const DEFAULT_SIZE: Address128 = 0x1000000;
    /// Page size: 4KB
    ///
    /// ---
    ///
    /// 页大小：4KB
    pub const PAGE_SIZE: Address128 = 0x1000;

    /// Creates a new memory system with the specified size.
    ///
    /// Creates a single read-write region from 0 to size.
    ///
    /// ---
    ///
    /// 创建指定大小的新内存系统。
    ///
    /// 创建从 0 到 size 的单个读写区域。
    pub fn new(size: Address128) -> Self {
        let mut tmp = Memory {
            size,
            regions: vec![],
            reservation_set: ReservationSet::new(),
        };
        tmp.add_region(0, size, MemoryPermissions::ReadWrite, String::from("main_memory"));
        tmp
    }

    /// Reads an 8-bit value from memory.
    ///
    /// ---
    ///
    /// 从内存读取 8 位值。
    pub fn read_8(&self, addr: Address128) -> Word8 {
        self.check_permissions(addr, MemoryPermissions::Read);
        let region = self.find_region(addr);
        if let Some(region) = region {
            let offset = addr - region.start;
            return region.data[offset as usize];
        }
        panic!("Memory access violation: {:x}", addr);
    }

    /// Reads a 16-bit value from memory (little-endian).
    ///
    /// ---
    ///
    /// 从内存读取 16 位值（小端）。
    pub fn read_16(&self, addr: Address128) -> Word16 {
        self.check_permissions(addr, MemoryPermissions::Read);
        let region = self.find_region(addr);
        if let Some(region) = region {
            let offset = addr - region.start;
            return (region.data[offset as usize] as Word16) |
                ((region.data[offset as usize + 1] as Word16)) << 8;
        }
        panic!("Memory access violation: {:x}", addr);
    }

    /// Reads a 32-bit value from memory (little-endian).
    ///
    /// ---
    ///
    /// 从内存读取 32 位值（小端）。
    pub fn read_32(&self, addr: Address128) -> Word32 {
        self.check_permissions(addr, MemoryPermissions::Read);
        let region = self.find_region(addr);
        if let Some(region) = region {
            let offset = addr - region.start;
            return (region.data[offset as usize] as Word32) |
                ((region.data[offset as usize + 1] as Word32)) << 8 |
                ((region.data[offset as usize + 2] as Word32)) << 16 |
                ((region.data[offset as usize + 3] as Word32)) << 24;
        }
        panic!("Memory access violation: {:x}", addr);
    }

    /// Reads a 64-bit value from memory (little-endian).
    ///
    /// ---
    ///
    /// 从内存读取 64 位值（小端）。
    pub fn read_64(&self, addr: Address128) -> Word64 {
        self.check_permissions(addr, MemoryPermissions::Read);
        let region = self.find_region(addr);
        if let Some(region) = region {
            let offset = addr - region.start;
            return (region.data[offset as usize] as Word64) |
                ((region.data[offset as usize + 1] as Word64)) << 8 |
                ((region.data[offset as usize + 2] as Word64)) << 16 |
                ((region.data[offset as usize + 3] as Word64)) << 24 |
                ((region.data[offset as usize + 4] as Word64)) << 32 |
                ((region.data[offset as usize + 5] as Word64)) << 40 |
                ((region.data[offset as usize + 6] as Word64)) << 48 |
                ((region.data[offset as usize + 7] as Word64)) << 56;
        }
        panic!("Memory access violation: {:x}", addr);
    }

    /// Reads a 128-bit value from memory (little-endian).
    ///
    /// ---
    ///
    /// 从内存读取 128 位值（小端）。
    pub fn read_128(&self, addr: Address128) -> Word128 {
        self.check_permissions(addr, MemoryPermissions::Read);
        let region = self.find_region(addr);
        if let Some(region) = region {
            let offset = addr - region.start;
            return (region.data[offset as usize] as Word128) |
                ((region.data[offset as usize + 1] as Word128)) << 8 |
                ((region.data[offset as usize + 2] as Word128)) << 16 |
                ((region.data[offset as usize + 3] as Word128)) << 24 |
                ((region.data[offset as usize + 4] as Word128)) << 32 |
                ((region.data[offset as usize + 5] as Word128)) << 40 |
                ((region.data[offset as usize + 6] as Word128)) << 48 |
                ((region.data[offset as usize + 7] as Word128)) << 56 |
                ((region.data[offset as usize + 8] as Word128)) << 64 |
                ((region.data[offset as usize + 9] as Word128)) << 72 |
                ((region.data[offset as usize + 10] as Word128)) << 80 |
                ((region.data[offset as usize + 11] as Word128)) << 88 |
                ((region.data[offset as usize + 12] as Word128)) << 96 |
                ((region.data[offset as usize + 13] as Word128)) << 104 |
                ((region.data[offset as usize + 14] as Word128)) << 112 |
                ((region.data[offset as usize + 15] as Word128)) << 120;
        }
        panic!("Memory access violation: {:x}", addr);
    }

    /// Writes an 8-bit value to memory.
    ///
    /// ---
    ///
    /// 将 8 位值写入内存。
    pub fn write_8(&mut self, addr: Address128, value: Word8) {
        self.check_permissions(addr, MemoryPermissions::Write);
        let region = self.find_region_mut(addr);
        if let Some(region) = region {
            let offset = addr - region.start;
            region.data[offset as usize] = value;
            return;
        }
        panic!("Memory access violation: {:x}", addr);
    }

    /// Writes a 16-bit value to memory (little-endian).
    ///
    /// ---
    ///
    /// 将 16 位值写入内存（小端）。
    pub fn write_16(&mut self, addr: Address128, value: Word16) {
        self.check_permissions(addr, MemoryPermissions::Write);
        let region = self.find_region_mut(addr);
        if let Some(region) = region {
            let offset = addr - region.start;
            region.data[offset as usize] = (value & 0xFF) as Word8;
            region.data[offset as usize + 1] = ((value >> 8) & 0xFF) as Word8;
            return;
        }
        panic!("Memory access violation: {:x}", addr);
    }

    /// Writes a 32-bit value to memory (little-endian).
    ///
    /// ---
    ///
    /// 将 32 位值写入内存（小端）。
    pub fn write_32(&mut self, addr: Address128, value: Word32) {
        self.check_permissions(addr, MemoryPermissions::Write);
        let region = self.find_region_mut(addr);
        if let Some(region) = region {
            let offset = addr - region.start;
            region.data[offset as usize] = (value & 0xFF) as Word8;
            region.data[offset as usize + 1] = ((value >> 8) & 0xFF) as Word8;
            region.data[offset as usize + 2] = ((value >> 16) & 0xFF) as Word8;
            region.data[offset as usize + 3] = ((value >> 24) & 0xFF) as Word8;
            return;
        }
        panic!("Memory access violation: {:x}", addr);
    }

    /// Writes a 64-bit value to memory (little-endian).
    ///
    /// ---
    ///
    /// 将 64 位值写入内存（小端）。
    pub fn write_64(&mut self, addr: Address128, value: Word64) {
        self.check_permissions(addr, MemoryPermissions::Write);
        let region = self.find_region_mut(addr);
        if let Some(region) = region {
            let offset = addr - region.start;
            region.data[offset as usize] = (value & 0xFF) as Word8;
            region.data[offset as usize + 1] = ((value >> 8) & 0xFF) as Word8;
            region.data[offset as usize + 2] = ((value >> 16) & 0xFF) as Word8;
            region.data[offset as usize + 3] = ((value >> 24) & 0xFF) as Word8;
            region.data[offset as usize + 4] = ((value >> 32) & 0xFF) as Word8;
            region.data[offset as usize + 5] = ((value >> 40) & 0xFF) as Word8;
            region.data[offset as usize + 6] = ((value >> 48) & 0xFF) as Word8;
            region.data[offset as usize + 7] = ((value >> 56) & 0xFF) as Word8;
            return;
        }
        panic!("Memory access violation: {:x}", addr);
    }

    /// Writes a 128-bit value to memory (little-endian).
    ///
    /// ---
    ///
    /// 将 128 位值写入内存（小端）。
    pub fn write_128(&mut self, addr: Address128, value: Word128) {
        self.check_permissions(addr, MemoryPermissions::Write);
        let region = self.find_region_mut(addr);
        if let Some(region) = region {
            let offset = addr - region.start;
            region.data[offset as usize] = (value & 0xFF) as Word8;
            region.data[offset as usize + 1] = ((value >> 8) & 0xFF) as Word8;
            region.data[offset as usize + 2] = ((value >> 16) & 0xFF) as Word8;
            region.data[offset as usize + 3] = ((value >> 24) & 0xFF) as Word8;
            region.data[offset as usize + 4] = ((value >> 32) & 0xFF) as Word8;
            region.data[offset as usize + 5] = ((value >> 40) & 0xFF) as Word8;
            region.data[offset as usize + 6] = ((value >> 48) & 0xFF) as Word8;
            region.data[offset as usize + 7] = ((value >> 56) & 0xFF) as Word8;
            region.data[offset as usize + 8] = ((value >> 64) & 0xFF) as Word8;
            region.data[offset as usize + 9] = ((value >> 72) & 0xFF) as Word8;
            region.data[offset as usize + 10] = ((value >> 80) & 0xFF) as Word8;
            region.data[offset as usize + 11] = ((value >> 88) & 0xFF) as Word8;
            region.data[offset as usize + 12] = ((value >> 96) & 0xFF) as Word8;
            region.data[offset as usize + 13] = ((value >> 104) & 0xFF) as Word8;
            region.data[offset as usize + 14] = ((value >> 112) & 0xFF) as Word8;
            region.data[offset as usize + 15] = ((value >> 120) & 0xFF) as Word8;
            return;
        }
        panic!("Memory access violation: {:x}", addr);
    }

    /// Reads multiple bytes into a buffer.
    ///
    /// ---
    ///
    /// 将多个字节读入缓冲区。
    pub fn read_bytes(&self, addr: Address128, buffer: &mut [Word8], size: usize) {
        for i in 0..size {
            buffer[i] = self.read_8(addr + i as Address128);
        }
    }

    /// Writes multiple bytes from a buffer.
    ///
    /// ---
    ///
    /// 从缓冲区写入多个字节。
    pub fn write_bytes(&mut self, addr: Address128, buffer: &[Word8], size: usize) {
        for i in 0..size {
            self.write_8(addr + i as Address128, buffer[i]);
        }
    }

    /// Adds a new memory region.
    ///
    /// ---
    ///
    /// 添加新内存区域。
    pub fn add_region(&mut self, start: Address128, size: Address128,
                      perms: MemoryPermissions, name: String){
        self.regions.push(MemoryRegion::new(start, size, perms, name));
    }

    /// Removes a memory region by start address.
    ///
    /// ---
    ///
    /// 按起始地址移除内存区域。
    pub fn remove_region(&mut self, start: Address128) {
        for i in 0..self.regions.len() {
            if self.regions[i].start == start {
                self.regions.remove(i);
                return;
            }
        }
    }

    /// Checks if an address belongs to a memory region.
    ///
    /// ---
    ///
    /// 检查地址是否属于某个内存区域。
    pub fn has_region(&self, addr: Address128) -> bool {
        return self.find_region(addr).is_some();
    }

    /// Returns permissions for an address.
    ///
    /// ---
    ///
    /// 返回地址的权限。
    pub fn get_permissions(&self, addr: Address128) -> MemoryPermissions {
        let region = self.find_region(addr);
        if let Some(region) = region {
            return region.permissions;
        }
        MemoryPermissions::None
    }

    /// Resets all memory to zero.
    ///
    /// ---
    ///
    /// 将所有内存重置为零。
    pub fn reset(&mut self) {
        for region in &mut self.regions {
            region.data.fill(0);
        }
    }

    /// Returns total memory size.
    ///
    /// ---
    ///
    /// 返回总内存大小。
    pub fn get_size(&self) -> Address128 {
        self.size
    }

    /// Checks if an address is valid.
    ///
    /// ---
    ///
    /// 检查地址是否有效。
    pub fn is_valid_address(&self, addr: Address128) -> bool {
        self.find_region(addr).is_some()
    }

    fn find_region(&self, addr: Address128) -> Option<&MemoryRegion> {
        for region in &self.regions {
            if addr >= region.start && addr < region.end {
                return Some(region);
            }
        }
        None
    }

    fn find_region_mut(&mut self, addr: Address128) -> Option<&mut MemoryRegion> {
        for region in &mut self.regions {
            if addr >= region.start && addr < region.end {
                return Some(region);
            }
        }
        None
    }
    fn check_permissions(&self, addr: Address128, required: MemoryPermissions) {
        let permissions = self.get_permissions(addr);
        if !permissions.contains(required) {
            panic!("Memory access violation: {:x} does not have required permissions {:?}", addr, required);
        }
    }

    // ========================================
    // Atomic Memory Operations (A Extension)
    // 原子内存操作（A 扩展）
    // ========================================

    /// Load-Reserved for 128-bit value.
    ///
    /// Creates a reservation at the given address and returns the value.
    /// Used for LR.D instruction.
    ///
    /// ---
    ///
    /// 128 位加载保留。
    ///
    /// 在给定地址创建保留并返回值。
    /// 用于 LR.D 指令。
    pub fn load_reserved_128(&mut self, addr: Address128) -> Word128 {
        let value = self.read_128(addr);
        self.reservation_set.set_reservation(addr, 16);
        value
    }

    /// Store-Conditional for 128-bit value.
    ///
    /// Stores the value if the reservation is still valid.
    /// Returns 0 on success, 1 on failure.
    ///
    /// ---
    ///
    /// 128 位存储条件。
    ///
    /// 如果保留仍然有效则存储值。
    /// 成功返回 0，失败返回 1。
    pub fn store_conditional_128(&mut self, addr: Address128, value: Word128) -> u32 {
        if self.reservation_set.check_reservation(addr, 16) {
            self.write_128(addr, value);
            self.reservation_set.clear_reservation();
            0 // Success
        } else {
            1 // Failure
        }
    }

    /// Load-Reserved for 64-bit value.
    ///
    /// Creates a reservation at the given address and returns the value.
    ///
    /// ---
    ///
    /// 64 位加载保留。
    ///
    /// 在给定地址创建保留并返回值。
    pub fn load_reserved_64(&mut self, addr: Address128) -> Word64 {
        let value = self.read_64(addr);
        self.reservation_set.set_reservation(addr, 8);
        value
    }

    /// Store-Conditional for 64-bit value.
    ///
    /// Stores the value if the reservation is still valid.
    /// Returns 0 on success, 1 on failure.
    ///
    /// ---
    ///
    /// 64 位存储条件。
    ///
    /// 如果保留仍然有效则存储值。
    /// 成功返回 0，失败返回 1。
    pub fn store_conditional_64(&mut self, addr: Address128, value: Word64) -> u32 {
        if self.reservation_set.check_reservation(addr, 8) {
            self.write_64(addr, value);
            self.reservation_set.clear_reservation();
            0 // Success
        } else {
            1 // Failure
        }
    }

    /// Load-Reserved for 32-bit value.
    ///
    /// Creates a reservation at the given address and returns the value.
    ///
    /// ---
    ///
    /// 32 位加载保留。
    ///
    /// 在给定地址创建保留并返回值。
    pub fn load_reserved_32(&mut self, addr: Address128) -> Word32 {
        let value = self.read_32(addr);
        self.reservation_set.set_reservation(addr, 4);
        value
    }

    /// Store-Conditional for 32-bit value.
    ///
    /// Stores the value if the reservation is still valid.
    /// Returns 0 on success, 1 on failure.
    ///
    /// ---
    ///
    /// 32 位存储条件。
    ///
    /// 如果保留仍然有效则存储值。
    /// 成功返回 0，失败返回 1。
    pub fn store_conditional_32(&mut self, addr: Address128, value: Word32) -> u32 {
        if self.reservation_set.check_reservation(addr, 4) {
            self.write_32(addr, value);
            self.reservation_set.clear_reservation();
            0 // Success
        } else {
            1 // Failure
        }
    }

    /// Returns a reference to the reservation set.
    ///
    /// ---
    ///
    /// 返回保留集的引用。
    pub fn get_reservation_set(&self) -> &ReservationSet {
        &self.reservation_set
    }

    /// Returns a mutable reference to the reservation set.
    ///
    /// ---
    ///
    /// 返回保留集的可变引用。
    pub fn get_reservation_set_mut(&mut self) -> &mut ReservationSet {
        &mut self.reservation_set
    }

    // ========================================
    // AMO Operations (Atomic Memory Operations)
    // AMO 操作（原子内存操作）
    // ========================================

    /// Atomic Memory Operation: Add.
    ///
    /// Atomically adds rs2 to memory at addr and returns the original value.
    /// rd = MEM[addr]; MEM[addr] = MEM[addr] + rs2
    ///
    /// ---
    ///
    /// 原子内存操作：加法。
    ///
    /// 原子地将 rs2 加到内存地址 addr，并返回原始值。
    /// rd = MEM[addr]; MEM[addr] = MEM[addr] + rs2
    pub fn amo_add_128(&mut self, addr: Address128, value: Word128) -> Word128 {
        let original = self.read_128(addr);
        self.write_128(addr, original.wrapping_add(value));
        original
    }

    /// Atomic Memory Operation: Swap.
    ///
    /// Atomically swaps value with memory at addr and returns the original value.
    /// rd = MEM[addr]; MEM[addr] = rs2
    ///
    /// ---
    ///
    /// 原子内存操作：交换。
    ///
    /// 原子地将值与内存地址 addr 交换，并返回原始值。
    /// rd = MEM[addr]; MEM[addr] = rs2
    pub fn amo_swap_128(&mut self, addr: Address128, value: Word128) -> Word128 {
        let original = self.read_128(addr);
        self.write_128(addr, value);
        original
    }

    /// Atomic Memory Operation: Logical AND.
    ///
    /// Atomically ANDs rs2 with memory at addr and returns the original value.
    /// rd = MEM[addr]; MEM[addr] = MEM[addr] & rs2
    ///
    /// ---
    ///
    /// 原子内存操作：逻辑与。
    ///
    /// 原子地将 rs2 与内存地址 addr 进行 AND，并返回原始值。
    /// rd = MEM[addr]; MEM[addr] = MEM[addr] & rs2
    pub fn amo_and_128(&mut self, addr: Address128, value: Word128) -> Word128 {
        let original = self.read_128(addr);
        self.write_128(addr, original & value);
        original
    }

    /// Atomic Memory Operation: Logical OR.
    ///
    /// Atomically ORs rs2 with memory at addr and returns the original value.
    /// rd = MEM[addr]; MEM[addr] = MEM[addr] | rs2
    ///
    /// ---
    ///
    /// 原子内存操作：逻辑或。
    ///
    /// 原子地将 rs2 与内存地址 addr 进行 OR，并返回原始值。
    /// rd = MEM[addr]; MEM[addr] = MEM[addr] | rs2
    pub fn amo_or_128(&mut self, addr: Address128, value: Word128) -> Word128 {
        let original = self.read_128(addr);
        self.write_128(addr, original | value);
        original
    }

    /// Atomic Memory Operation: Logical XOR.
    ///
    /// Atomically XORs rs2 with memory at addr and returns the original value.
    /// rd = MEM[addr]; MEM[addr] = MEM[addr] ^ rs2
    ///
    /// ---
    ///
    /// 原子内存操作：逻辑异或。
    ///
    /// 原子地将 rs2 与内存地址 addr 进行 XOR，并返回原始值。
    /// rd = MEM[addr]; MEM[addr] = MEM[addr] ^ rs2
    pub fn amo_xor_128(&mut self, addr: Address128, value: Word128) -> Word128 {
        let original = self.read_128(addr);
        self.write_128(addr, original ^ value);
        original
    }

    /// Atomic Memory Operation: Maximum signed.
    ///
    /// Atomically stores the maximum of MEM[addr] and rs2, returns original.
    /// rd = MEM[addr]; MEM[addr] = max(MEM[addr], rs2) (signed)
    ///
    /// ---
    ///
    /// 原子内存操作：有符号最大值。
    ///
    /// 原子地存储 MEM[addr] 和 rs2 的最大值，返回原始值。
    /// rd = MEM[addr]; MEM[addr] = max(MEM[addr], rs2) (有符号)
    pub fn amo_max_128(&mut self, addr: Address128, value: Word128) -> Word128 {
        let original = self.read_128(addr);
        let original_signed = original as i128;
        let value_signed = value as i128;
        let result = if original_signed > value_signed { original_signed } else { value_signed };
        self.write_128(addr, result as Word128);
        original
    }

    /// Atomic Memory Operation: Maximum unsigned.
    ///
    /// Atomically stores the maximum of MEM[addr] and rs2, returns original.
    /// rd = MEM[addr]; MEM[addr] = max(MEM[addr], rs2) (unsigned)
    ///
    /// ---
    ///
    /// 原子内存操作：无符号最大值。
    ///
    /// 原子地存储 MEM[addr] 和 rs2 的最大值，返回原始值。
    /// rd = MEM[addr]; MEM[addr] = max(MEM[addr], rs2) (无符号)
    pub fn amo_maxu_128(&mut self, addr: Address128, value: Word128) -> Word128 {
        let original = self.read_128(addr);
        let result = if original > value { original } else { value };
        self.write_128(addr, result);
        original
    }

    /// Atomic Memory Operation: Minimum signed.
    ///
    /// Atomically stores the minimum of MEM[addr] and rs2, returns original.
    /// rd = MEM[addr]; MEM[addr] = min(MEM[addr], rs2) (signed)
    ///
    /// ---
    ///
    /// 原子内存操作：有符号最小值。
    ///
    /// 原子地存储 MEM[addr] 和 rs2 的最小值，返回原始值。
    /// rd = MEM[addr]; MEM[addr] = min(MEM[addr], rs2) (有符号)
    pub fn amo_min_128(&mut self, addr: Address128, value: Word128) -> Word128 {
        let original = self.read_128(addr);
        let original_signed = original as i128;
        let value_signed = value as i128;
        let result = if original_signed < value_signed { original_signed } else { value_signed };
        self.write_128(addr, result as Word128);
        original
    }

    /// Atomic Memory Operation: Minimum unsigned.
    ///
    /// Atomically stores the minimum of MEM[addr] and rs2, returns original.
    /// rd = MEM[addr]; MEM[addr] = min(MEM[addr], rs2) (unsigned)
    ///
    /// ---
    ///
    /// 原子内存操作：无符号最小值。
    ///
    /// 原子地存储 MEM[addr] 和 rs2 的最小值，返回原始值。
    /// rd = MEM[addr]; MEM[addr] = min(MEM[addr], rs2) (无符号)
    pub fn amo_minu_128(&mut self, addr: Address128, value: Word128) -> Word128 {
        let original = self.read_128(addr);
        let result = if original < value { original } else { value };
        self.write_128(addr, result);
        original
    }

    // ========================================
    // 64-bit AMO Operations / 64 位 AMO 操作
    // ========================================

    pub fn amo_add_64(&mut self, addr: Address128, value: Word64) -> Word64 {
        let original = self.read_64(addr);
        self.write_64(addr, original.wrapping_add(value));
        original
    }

    pub fn amo_swap_64(&mut self, addr: Address128, value: Word64) -> Word64 {
        let original = self.read_64(addr);
        self.write_64(addr, value);
        original
    }

    pub fn amo_and_64(&mut self, addr: Address128, value: Word64) -> Word64 {
        let original = self.read_64(addr);
        self.write_64(addr, original & value);
        original
    }

    pub fn amo_or_64(&mut self, addr: Address128, value: Word64) -> Word64 {
        let original = self.read_64(addr);
        self.write_64(addr, original | value);
        original
    }

    pub fn amo_xor_64(&mut self, addr: Address128, value: Word64) -> Word64 {
        let original = self.read_64(addr);
        self.write_64(addr, original ^ value);
        original
    }

    pub fn amo_max_64(&mut self, addr: Address128, value: Word64) -> Word64 {
        let original = self.read_64(addr);
        let original_signed = original as i64;
        let value_signed = value as i64;
        let result = if original_signed > value_signed { original_signed } else { value_signed };
        self.write_64(addr, result as Word64);
        original
    }

    pub fn amo_maxu_64(&mut self, addr: Address128, value: Word64) -> Word64 {
        let original = self.read_64(addr);
        let result = if original > value { original } else { value };
        self.write_64(addr, result);
        original
    }

    pub fn amo_min_64(&mut self, addr: Address128, value: Word64) -> Word64 {
        let original = self.read_64(addr);
        let original_signed = original as i64;
        let value_signed = value as i64;
        let result = if original_signed < value_signed { original_signed } else { value_signed };
        self.write_64(addr, result as Word64);
        original
    }

    pub fn amo_minu_64(&mut self, addr: Address128, value: Word64) -> Word64 {
        let original = self.read_64(addr);
        let result = if original < value { original } else { value };
        self.write_64(addr, result);
        original
    }

    // ========================================
    // 32-bit AMO Operations / 32 位 AMO 操作
    // ========================================

    pub fn amo_add_32(&mut self, addr: Address128, value: Word32) -> Word32 {
        let original = self.read_32(addr);
        self.write_32(addr, original.wrapping_add(value));
        original
    }

    pub fn amo_swap_32(&mut self, addr: Address128, value: Word32) -> Word32 {
        let original = self.read_32(addr);
        self.write_32(addr, value);
        original
    }

    pub fn amo_and_32(&mut self, addr: Address128, value: Word32) -> Word32 {
        let original = self.read_32(addr);
        self.write_32(addr, original & value);
        original
    }

    pub fn amo_or_32(&mut self, addr: Address128, value: Word32) -> Word32 {
        let original = self.read_32(addr);
        self.write_32(addr, original | value);
        original
    }

    pub fn amo_xor_32(&mut self, addr: Address128, value: Word32) -> Word32 {
        let original = self.read_32(addr);
        self.write_32(addr, original ^ value);
        original
    }

    pub fn amo_max_32(&mut self, addr: Address128, value: Word32) -> Word32 {
        let original = self.read_32(addr);
        let original_signed = original as i32;
        let value_signed = value as i32;
        let result = if original_signed > value_signed { original_signed } else { value_signed };
        self.write_32(addr, result as Word32);
        original
    }

    pub fn amo_maxu_32(&mut self, addr: Address128, value: Word32) -> Word32 {
        let original = self.read_32(addr);
        let result = if original > value { original } else { value };
        self.write_32(addr, result);
        original
    }

    pub fn amo_min_32(&mut self, addr: Address128, value: Word32) -> Word32 {
        let original = self.read_32(addr);
        let original_signed = original as i32;
        let value_signed = value as i32;
        let result = if original_signed < value_signed { original_signed } else { value_signed };
        self.write_32(addr, result as Word32);
        original
    }

    pub fn amo_minu_32(&mut self, addr: Address128, value: Word32) -> Word32 {
        let original = self.read_32(addr);
        let result = if original < value { original } else { value };
        self.write_32(addr, result);
        original
    }
}
