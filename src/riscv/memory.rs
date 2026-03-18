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
/// ---
///
/// 虚拟机的内存系统。
///
/// 管理多个具有不同权限的内存区域。
/// 默认配置创建单个读写区域。
pub struct Memory {
    regions: Vec<MemoryRegion>,
    size: Address128,
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
}
