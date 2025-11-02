#![no_std]
#![no_main]

use core::{arch::asm, ptr::NonNull};
use log::{error, info};
use uefi::{
    CString16,
    boot::{self, ScopedProtocol},
    fs::FileSystem,
    guid,
    prelude::*,
    proto::media::fs::SimpleFileSystem,
    system::with_config_table,
    table::cfg::ConfigTableEntry,
};

mod elf;

pub fn get_currentel() -> u64 {
    let currentel: u64;
    unsafe { asm!("mrs {}, currentel", out(reg) currentel) };
    currentel
}

fn boot(elf_base: usize, entry_point: usize, dtb_entry: &ConfigTableEntry) -> ! {
    let dtb_ptr = dtb_entry.address as *const u8;
    info!("Booting with DTB at address {:p}", dtb_ptr as *const u8);

    let argv: [*const u8; 3] = [dtb_ptr, elf_base as *const u8, core::ptr::null()];
    let argc = argv.len() - 1;

    // Jump to the ELF entry point with function signature `extern "C" fn main(argc: usize, argv: *const *const u8) -> usize`
    let entry_fn: extern "C" fn(usize, *const *const u8) -> ! =
        // SAFETY: We ensure that the entry_point is a valid function pointer.
        unsafe { core::mem::transmute(entry_point) };
    entry_fn(argc, argv.as_ptr());
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("MiniVisor UEFI Loader");

    let current_el = get_currentel() >> 2 & 0b11;
    if current_el != 2 {
        error!("Current EL must be EL2");
        return Status::UNSUPPORTED;
    }
    info!("Current EL: {:#x}", current_el);

    let path: CString16 = CString16::try_from("mini_visor").unwrap();
    let fs: ScopedProtocol<SimpleFileSystem> =
        boot::get_image_file_system(boot::image_handle()).unwrap();
    let mut fs = FileSystem::new(fs);
    if !fs.try_exists(path.as_ref()).unwrap() {
        error!("mini_visor not found");
        return Status::NOT_FOUND;
    }
    let binary = fs.read(path.as_ref()).unwrap();
    info!("Read mini_visor, size {}", binary.len());

    let mut elf_base: Option<usize> = None;
    let elf_header = elf::Elf64Header::new(binary.as_ptr() as usize).expect("Invalid ELF Header");
    for p in elf_header.get_program_headers() {
        if p.get_segment_type() == elf::ELF_PROGRAM_HEADER_SEGMENT_LOAD {
            let phys_addr = p.get_physical_address();
            let mem_size = p.get_memory_size();
            let offset = p.get_offset();
            let file_size = p.get_file_size();

            // Assume the first LOAD segment's physical address as the ELF base address
            if elf_base.is_none() {
                elf_base = Some(phys_addr as usize);
            }

            info!(
                "Loading segment: phys_addr={:#x}, mem_size={:#x}, offset={:#x}, file_size={:#x}",
                phys_addr, mem_size, offset, file_size
            );

            if offset + file_size > binary.len() as u64 {
                error!("Segment exceeds binary size");
                return Status::LOAD_ERROR;
            }

            if mem_size < file_size {
                error!("Segment memory size is smaller than file size");
                return Status::LOAD_ERROR;
            }

            const PAGE_SIZE: u64 = 0x1000;
            let num_pages = (mem_size + PAGE_SIZE - 1) / PAGE_SIZE;

            let allocated_pages = boot::allocate_pages(
                boot::AllocateType::Address(phys_addr),
                boot::MemoryType::LOADER_CODE,
                num_pages as usize,
            )
            .expect("Failed to allocate pages for the segment");

            let src_ptr =
                NonNull::new((binary.as_ptr() as usize + offset as usize) as *mut u8).unwrap();

            // SAFETY: We have allocated enough pages for the segment.
            unsafe {
                allocated_pages.copy_from(src_ptr, file_size as usize);
            }
        }
    }

    let elf_base = elf_base.expect("Failed to find ELF base address");

    let entry_point = elf_header.get_entry_point() as usize;
    info!("ELF entry point at address {:#x}", entry_point);

    const DTB_TABLE_GUID: uefi::Guid = guid!("B1B621D5-F19C-41A5-830B-D9152C69AAE0");

    with_config_table(|slice| {
        for i in slice {
            if let DTB_TABLE_GUID = i.guid {
                info!(
                    "Found DTB table entry at {:p}",
                    i as *const ConfigTableEntry
                );
                boot(elf_base, entry_point, i);
            }
        }
    });

    info!("DTB table not found");

    Status::NOT_FOUND
}
