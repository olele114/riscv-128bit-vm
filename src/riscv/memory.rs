use bitflags::bitflags;

type Address128 = u128;
type Word128 = u128;
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

    pub fn add_region(&mut self, start: Address128, size: Address128,
                      perms: MemoryPermissions, name: String){
        self.regions.push(MemoryRegion::new(start, size, perms, name));
    }

    pub fn get_permissions(&self, addr: Address128) -> MemoryPermissions {
        let region = self.find_region(addr);
        if let Some(region) = region {
            return region.permissions;
        }
        MemoryPermissions::None
    }

    fn find_region(&self, addr: Address128) -> Option<&MemoryRegion> {
        for region in &self.regions {
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
