#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
mod serial;
mod asm;
mod console;
mod dtb;
mod drivers {
    pub mod generic_timer;
    pub mod gicv3;
    pub mod virtio;
}
mod elf;
mod exception;
mod lock;
mod mmio {
    pub mod gicv3;
    pub mod pl011;
    pub mod virtio_blk;
}
mod paging;
mod psci;
mod registers;
mod vgic;
mod vm;

use uefi::entry;

use drivers::{generic_timer, gicv3};
use lock::Mutex;
use psci::PsciErrorCodes;
use serial::SerialDevice;

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

/// グローバル変数置き場
static CONSOLE: Mutex<console::Console> = Mutex::new(console::Console::new());
static IS_CONSOLE_ACTIVE: AtomicBool = AtomicBool::new(false);
static mut DTB: MaybeUninit<dtb::Dtb> = MaybeUninit::uninit();

/// 定数
const STACK_SIZE: usize = 0x10000;
const CONSOLE_SWITCH_KEY: u8 = 0x13; /* Ctrl + S */

#[entry]
fn main() -> uefi::Status {
    uefi::helpers::init().unwrap();

    const DTB_TABLE_GUID: uefi::Guid = uefi::guid!("B1B621D5-F19C-41A5-830B-D9152C69AAE0");

    let mut dtb: Option<dtb::Dtb> = None;
    uefi::system::with_config_table(|slice| {
        for i in slice {
            if let DTB_TABLE_GUID = i.guid {
                log::info!(
                    "Found DTB table entry at {:p}",
                    i as *const uefi::table::cfg::ConfigTableEntry
                );
                dtb = Some(dtb::Dtb::new(i.address as usize).unwrap());
                break;
            }
        }
    });

    let dtb = dtb.expect("DTB is not found in the UEFI Config Table");

    println!("Hello, world!");

    let current_el = asm::get_currentel() >> 2;
    println!("CurrentEL: {}", current_el);
    assert_eq!(current_el, 2);

    exception::setup_exception();
    let _distributor = init_gic_distributor(&dtb);
    let redistributor = init_gic_redistributor(&dtb);

    generic_timer::init_generic_timer_global(&dtb);

    let (boot_address, argument) = vm::create_vm(&redistributor);

    /* PSCIのバージョンチェック */
    let (major_version, minor_version) = psci::check_psci_version().expect("PSCI is not supported");
    println!("PSCI version {major_version}.{minor_version}");

    vm::boot_vm(boot_address, argument)
}

fn str_to_usize(s: &str) -> Option<usize> {
    let radix;
    let start;
    match s.get(0..2) {
        Some("0x") => {
            radix = 16;
            start = s.get(2..);
        }
        Some("0o") => {
            radix = 8;
            start = s.get(2..);
        }
        Some("0b") => {
            radix = 2;
            start = s.get(2..);
        }
        _ => {
            radix = 10;
            start = Some(s);
        }
    }
    usize::from_str_radix(start?, radix).ok()
}

fn init_gic_distributor(dtb: &dtb::Dtb) -> gicv3::GicDistributor {
    let gic_node = dtb.search_node_by_compatible(b"arm,gic-v3", None).unwrap();
    let (base_address, size) = dtb.read_reg_property(&gic_node, 0).unwrap();
    println!("GIC Distributor's Base Address: {:#X}", base_address);
    let gic_distributor = gicv3::GicDistributor::new(base_address, size).unwrap();
    gic_distributor.init();
    gic_distributor
}

fn init_gic_redistributor(dtb: &dtb::Dtb) -> gicv3::GicRedistributor {
    let gic_node = dtb.search_node_by_compatible(b"arm,gic-v3", None).unwrap();
    let (base_address, size) = dtb.read_reg_property(&gic_node, 1).unwrap();
    println!("GIC Redistributor's Base Address: {:#X}", base_address);
    let gic_redistributor = gicv3::get_self_redistributor(base_address, size).unwrap();
    gic_redistributor.init();
    gic_redistributor
}

pub fn launch_cpu() -> bool {
    let dtb = unsafe { (&raw const DTB).as_ref().unwrap().assume_init_ref() };
    let mut cpu_node = None;
    let current_affinity = asm::mpidr_to_affinity(asm::get_mpidr_el1());
    let stack_base = uefi::boot::allocate_pages(
        uefi::boot::AllocateType::AnyPages,
        uefi::boot::MemoryType::LOADER_DATA,
        STACK_SIZE >> paging::PAGE_SHIFT,
    )
    .expect("Failed to allocate memory");
    let stack_address = stack_base.addr().get() + STACK_SIZE;

    /* Copy registers controlling the paging */
    use core::ptr::write_volatile;
    unsafe {
        write_volatile((stack_address - 8 * 4) as *mut u64, asm::get_tcr_el2());
        write_volatile((stack_address - 8 * 3) as *mut u64, asm::get_ttbr0_el2());
        write_volatile((stack_address - 8 * 2) as *mut u64, asm::get_mair_el2());
        write_volatile((stack_address - 8) as *mut u64, asm::get_sctlr_el2());
    }
    asm::flush_data_cache_all();

    while let Some(cpu) = dtb.search_node(b"cpu", cpu_node.as_ref()) {
        if let Some((affinity, _)) = dtb.read_reg_property(&cpu, 0)
            && current_affinity != affinity as u64
        {
            match psci::cpu_on(
                affinity as u64,
                asm::core_entry as *const fn() as usize as u64,
                stack_address as u64,
            ) {
                Ok(_) => return true,
                Err(PsciErrorCodes::AlreadyOn) => { /* 次のノードを探索 */ }
                Err(e) => {
                    println!("Failed to start CPU(Affinity: {:#X}): {:?}", affinity, e);
                }
            }
        }
        cpu_node = Some(cpu);
    }
    unsafe {
        let _ = uefi::boot::free_pages(stack_base, STACK_SIZE >> paging::PAGE_SHIFT);
    };
    false
}

extern "C" fn core_main() -> ! {
    let current_el = asm::get_currentel() >> 2;
    assert_eq!(current_el, 2);

    exception::setup_exception();
    let redistributor =
        init_gic_redistributor(unsafe { (&raw const DTB).as_ref().unwrap().assume_init_ref() });

    let (boot_address, argument) = vm::create_vm(&redistributor);
    vm::boot_vm(boot_address, argument)
}

#[allow(dead_code)]
fn handle_input(device: &Mutex<dyn SerialDevice>) {
    loop {
        let c = device.lock().getc();
        if c.is_err() {
            println!("Failed to get a character");
            return;
        }
        let c = c.unwrap().unwrap_or(0);
        if c == 0 {
            return;
        }
        if c == CONSOLE_SWITCH_KEY {
            let old = IS_CONSOLE_ACTIVE.fetch_xor(true, Ordering::Relaxed);
            if old {
                /* コンソール無効化: プロンプトを上書き */
                print!("\r");
            } else {
                /* コンソール有効化: プロンプトを出力 */
                CONSOLE.lock().reset_buffer();
            }
        } else if IS_CONSOLE_ACTIVE.load(Ordering::Relaxed) {
            CONSOLE.lock().write(c);
        } else {
            vm::input_uart(c);
        }
    }
}
