#![no_std]
#![no_main]

use core::arch::asm;
use core::time::Duration;
use log::info;
use uefi::guid;
use uefi::prelude::*;
use uefi::{system::with_config_table, table::cfg::ConfigTableEntry};

pub fn get_currentel() -> u64 {
    let currentel: u64;
    unsafe { asm!("mrs {}, currentel", out(reg) currentel) };
    currentel
}

fn boot(dtb_entry: &ConfigTableEntry) -> ! {
    let dtb_ptr = dtb_entry.address as *const u8;
    info!("Booting with DTB at address {:p}", dtb_ptr as *const u8);
    // TODO: Determine the actual size of the DTB.
    // SAFETY: We assume the DTB pointer is valid.
    let dtb_slice = unsafe { core::slice::from_raw_parts(dtb_ptr, 0x10000) };
    info!(
        "DTB slice at {:p}, length {}",
        dtb_slice.as_ptr(),
        dtb_slice.len()
    );
    loop {}
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Hello world!");

    let current_el = get_currentel() >> 2 & 0b11;
    info!("Current EL: {:#x}", current_el);

    const DTB_TABLE_GUID: uefi::Guid = guid!("B1B621D5-F19C-41A5-830B-D9152C69AAE0");

    with_config_table(|slice| {
        for i in slice {
            if let DTB_TABLE_GUID = i.guid {
                info!(
                    "Found DTB table entry at {:p}",
                    i as *const ConfigTableEntry
                );
                boot(i);
            }
        }
    });

    info!("DTB table not found");

    Status::SUCCESS
}
