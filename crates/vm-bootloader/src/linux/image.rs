use std::fs;
use std::path::Path;

use anyhow::anyhow;
use anyhow::ensure;
use vm_core::mm::allocator::MemoryContainer;
use vm_core::mm::manager::MemoryAddressSpace;
use vm_core::vcpu::arch::aarch64::AArch64Vcpu;
use vm_core::virt::Virt;
use zerocopy::FromBytes;

use crate::BootLoader;

mod layout {}

#[repr(C)]
#[derive(FromBytes)]
struct Header {
    code0: u32,
    code1: u32,
    text_offset: u64,
    image_size: u64,
    flags: u64,
    res2: u64,
    res3: u64,
    res4: u64,
    magic: u32,
    res5: u32,
}

#[allow(dead_code)]
pub struct Image {
    image: Vec<u8>,
    initrd: Option<Vec<u8>>,
    cmdline: Option<String>,
}

impl Image {
    pub fn new(
        kernel: &Path,
        initrd: Option<&Path>,
        cmdline: Option<&str>,
    ) -> anyhow::Result<Self> {
        let image = fs::read(kernel)?;
        let initrd = initrd.map(fs::read).transpose()?;
        let cmdline = cmdline.map(|s| s.to_string());

        Ok(Image {
            image,
            initrd,
            cmdline,
        })
    }

    fn validate(&self) -> anyhow::Result<()> {
        let len = size_of::<Header>();

        ensure!(self.image.len() >= len, "image too small");

        let header = Header::read_from_bytes(&self.image[0..len])
            .map_err(|_| anyhow!("Invalid image header"))?;

        ensure!(header.magic == 0x644d5241, "Invalid header magic");

        Ok(())
    }

    fn setup_memory<C>(&self, _memory: &mut MemoryAddressSpace<C>) -> anyhow::Result<()>
    where
        C: MemoryContainer,
    {
        todo!()
    }

    fn setup_device_tree(&self) -> anyhow::Result<()> {
        todo!()
    }

    fn setup_boot_cpu<V>(&self, _boot_cpu: &mut V) -> anyhow::Result<()>
    where
        V: AArch64Vcpu,
    {
        todo!()
    }
}

impl<V> BootLoader<V> for Image
where
    V: Virt,
    V::Vcpu: AArch64Vcpu,
{
    fn install(
        &self,
        memory: &mut MemoryAddressSpace<V::Memory>,
        _memory_size: usize,
        boot_cpu: &mut V::Vcpu,
    ) -> anyhow::Result<()> {
        self.validate()?;

        self.setup_memory(memory)?;

        self.setup_device_tree()?;

        self.setup_boot_cpu(boot_cpu)?;

        Ok(())
    }
}
