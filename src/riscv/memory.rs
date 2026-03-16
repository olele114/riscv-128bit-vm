use bitflags::bitflags;

pub(crate) type Address128 = u128;
pub(crate) type Word128 = u128;
type Word64 = u64;
type Word32 = u32;
type Word16 = u16;
type Word8 = u8;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct MemoryPermissions : u8 {
        const None = 0;
        const Read = 1;
        const Write = 2;
        const Execute = 4;

        const ReadWrite = Self::Read.bits() | Self::Write.bits();
        const ReadExecute = Self::Read.bits() | Self::Execute.bits();
        const All = Self::ReadWrite.bits() | Self::Execute.bits();
    }
}

struct MemoryRegion {
    start: Address128,
    end: Address128,
    data: Vec<u8>,
    permissions: MemoryPermissions,
    name: String,
}

pub struct Memory {
    regions: Vec<MemoryRegion>,
    size: Address128,
}

impl MemoryRegion {
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
    pub const DEFAULT_SIZE: Address128 = 0x1000000; // 16MB default
    pub const PAGE_SIZE: Address128 = 0x1000; // 4KB pages

    pub fn new(size: Address128) -> Self {
        let mut tmp = Memory {
            size,
            regions: vec![],
        };
        tmp.add_region(0, size, MemoryPermissions::ReadWrite, String::from("main_memory"));
        tmp
    }

    pub fn read_8(&self, addr: Address128) -> Word8 {
        self.check_permissions(addr, MemoryPermissions::Read);
        let region = self.find_region(addr);
        if let Some(region) = region {
            let offset = addr - region.start;
            return region.data[offset as usize];
        }
        panic!("Memory access violation: {:x}", addr);
    }

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

    pub fn read_bytes(&self, addr: Address128, buffer: &mut [Word8], size: usize) {
        for i in 0..size {
            buffer[i] = self.read_8(addr + i as Address128);
        }
    }

    pub fn write_bytes(&mut self, addr: Address128, buffer: &[Word8], size: usize) {
        for i in 0..size {
            self.write_8(addr + i as Address128, buffer[i]);
        }
    }

    pub fn add_region(&mut self, start: Address128, size: Address128,
                      perms: MemoryPermissions, name: String){
        self.regions.push(MemoryRegion::new(start, size, perms, name));
    }

    pub fn remove_region(&mut self, start: Address128) {
        for i in 0..self.regions.len() {
            if self.regions[i].start == start {
                self.regions.remove(i);
                return;
            }
        }
    }

    pub fn has_region(&self, addr: Address128) -> bool {
        return self.find_region(addr).is_some();
    }

    pub fn get_permissions(&self, addr: Address128) -> MemoryPermissions {
        let region = self.find_region(addr);
        if let Some(region) = region {
            return region.permissions;
        }
        MemoryPermissions::None
    }

    pub fn reset(&mut self) {
        for region in &mut self.regions {
            region.data.fill(0);
        }
    }

    pub fn get_size(&self) -> Address128 {
        self.size
    }

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
