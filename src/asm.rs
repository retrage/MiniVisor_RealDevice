//!
//! アセンブリを記述したモジュール
//!

use core::arch::{asm, naked_asm};

pub fn get_currentel() -> u64 {
    let currentel: u64;
    unsafe { asm!("mrs {}, currentel", out(reg) currentel) };
    currentel
}

pub unsafe fn set_hcr_el2(hcr_el2: u64) {
    unsafe { asm!("msr hcr_el2, {}", in(reg) hcr_el2) };
}

pub unsafe fn set_elr_el2(elr_el2: u64) {
    unsafe { asm!("msr elr_el2, {}", in(reg) elr_el2) };
}

pub unsafe fn set_spsr_el2(spsr_el2: u64) {
    unsafe { asm!("msr spsr_el2, {}", in(reg) spsr_el2) };
}

pub unsafe fn eret(x0: u64, x1: u64, x2: u64, x3: u64) -> ! {
    unsafe {
        asm!("eret",
             in("x0") x0,
             in("x1") x1,
             in("x2") x2,
             in("x3") x3,
             options(noreturn))
    }
}

pub fn get_stack_pointer() -> u64 {
    let sp: u64;
    unsafe { asm!("mov {}, sp", out(reg) sp) };
    sp
}

pub fn get_id_aa64mmfr0_el1() -> u64 {
    let id_aa64mmfr0_el1: u64;
    unsafe { asm!("mrs {}, id_aa64mmfr0_el1", out(reg) id_aa64mmfr0_el1) };
    id_aa64mmfr0_el1
}

pub fn get_sctlr_el2() -> u64 {
    let sctlr_el2: u64;
    unsafe { asm!("mrs {}, sctlr_el2", out(reg) sctlr_el2) };
    sctlr_el2
}

pub fn get_tcr_el2() -> u64 {
    let tcr_el2: u64;
    unsafe { asm!("mrs {}, tcr_el2", out(reg) tcr_el2) };
    tcr_el2
}

pub fn get_ttbr0_el2() -> u64 {
    let ttbr0_el2: u64;
    unsafe { asm!("mrs {}, ttbr0_el2", out(reg) ttbr0_el2) };
    ttbr0_el2
}

pub fn get_mair_el2() -> u64 {
    let mair_el2: u64;
    unsafe { asm!("mrs {}, mair_el2", out(reg) mair_el2) };
    mair_el2
}

pub fn get_vtcr_el2() -> u64 {
    let vtcr_el2: u64;
    unsafe { asm!("mrs {}, vtcr_el2", out(reg) vtcr_el2) };
    vtcr_el2
}

pub unsafe fn set_vtcr_el2(vtcr_el2: u64) {
    unsafe { asm!("msr vtcr_el2, {}", in(reg) vtcr_el2) };
}

pub fn get_vttbr_el2() -> u64 {
    let vttbr_el2: u64;
    unsafe { asm!("mrs {}, vttbr_el2", out(reg) vttbr_el2) };
    vttbr_el2
}

pub unsafe fn set_vttbr_el2(vttbr_el2: u64) {
    unsafe { asm!("msr vttbr_el2, {}", in(reg) vttbr_el2) };
}

pub fn flush_tlb_el1() {
    unsafe {
        asm!(
            "
            dsb ishst
            tlbi alle1is
            "
        );
    }
}

pub unsafe fn set_vbar_el2(vbar_el2: u64) {
    unsafe { asm!("msr vbar_el2, {}", in(reg) vbar_el2) };
}

pub fn get_elr_el2() -> u64 {
    let elr_el2: u64;
    unsafe { asm!("mrs {}, elr_el2", out(reg) elr_el2) };
    elr_el2
}

pub unsafe fn advance_elr_el2() {
    unsafe { set_elr_el2(get_elr_el2() + 4) }
}

pub fn get_esr_el2() -> u64 {
    let esr_el2: u64;
    unsafe { asm!("mrs {}, esr_el2", out(reg) esr_el2) };
    esr_el2
}

pub fn get_far_el2() -> u64 {
    let far_el2: u64;
    unsafe { asm!("mrs {}, far_el2", out(reg) far_el2) };
    far_el2
}

pub fn get_hpfar_el2() -> u64 {
    let hpfar_el2: u64;
    unsafe { asm!("mrs {}, hpfar_el2", out(reg) hpfar_el2) };
    hpfar_el2
}

pub fn get_mpidr_el1() -> u64 {
    let mpidr_el1: u64;
    unsafe { asm!("mrs {}, mpidr_el1", out(reg) mpidr_el1) };
    mpidr_el1
}

pub const fn mpidr_to_affinity(mpidr: u64) -> u64 {
    mpidr & !((1 << 31) | (1 << 30))
}

pub fn get_packed_affinity() -> u32 {
    let mpidr = mpidr_to_affinity(get_mpidr_el1());
    ((mpidr & ((1 << 24) - 1)) | ((mpidr & (0xff << 32)) >> (32 - 24))) as u32
}

pub fn get_icc_sre_el2() -> u64 {
    let icc_sre_el2: u64;
    unsafe { asm!("mrs {}, icc_sre_el2", out(reg) icc_sre_el2) };
    icc_sre_el2
}

pub unsafe fn set_icc_sre_el2(icc_sre_el2: u64) {
    unsafe { asm!("msr icc_sre_el2, {}", in(reg) icc_sre_el2) };
}

pub unsafe fn set_icc_igrpen1_el1(icc_igrpen1_el1: u64) {
    unsafe { asm!("msr icc_igrpen1_el1, {}", in(reg) icc_igrpen1_el1) };
}

pub unsafe fn set_icc_pmr_el1(icc_pmr_el1: u64) {
    unsafe { asm!("msr icc_pmr_el1, {}", in(reg) icc_pmr_el1) };
}

pub unsafe fn set_icc_bpr1_el1(icc_bpr1_el1: u64) {
    unsafe { asm!("msr icc_bpr1_el1, {}", in(reg) icc_bpr1_el1) };
}

pub unsafe fn set_icc_eoir1_el1(icc_eoir1_el1: u64) {
    unsafe { asm!("msr icc_eoir1_el1, {}", in(reg) icc_eoir1_el1) };
}

pub fn get_icc_iar1_el1() -> u64 {
    let icc_iar1_el1: u64;
    unsafe { asm!("mrs {}, icc_iar1_el1", out(reg) icc_iar1_el1) };
    icc_iar1_el1
}

pub unsafe fn invalidate_cache(address: usize) {
    unsafe { asm!("dc ivac, {}", in(reg) address) };
}

pub fn get_midr_el1() -> u64 {
    let midr_el1: u64;
    unsafe { asm!("mrs {}, midr_el1", out(reg) midr_el1) };
    midr_el1
}

pub unsafe fn set_vmpidr_el2(vmpidr_el2: u64) {
    unsafe { asm!("msr vmpidr_el2, {}", in(reg) vmpidr_el2) };
}

pub unsafe fn set_vpidr_el2(vpidr_el2: u64) {
    unsafe { asm!("msr vpidr_el2, {}", in(reg) vpidr_el2) };
}

pub unsafe fn set_icc_sgi1r_el1(icc_sgi1r_el1: u64) {
    unsafe { asm!("msr icc_sgi1r_el1, {}", in(reg) icc_sgi1r_el1) };
}

pub fn get_ich_eisr_el2() -> u64 {
    let ich_eisr_el2: u64;
    unsafe { asm!("mrs {}, ich_eisr_el2", out(reg) ich_eisr_el2) };
    ich_eisr_el2
}

pub fn get_ich_vtr_el2() -> u64 {
    let ich_vtr_el2: u64;
    unsafe { asm!("mrs {}, ich_vtr_el2", out(reg) ich_vtr_el2) };
    ich_vtr_el2
}

pub unsafe fn set_ich_hcr_el2(ich_hcr_el2: u64) {
    unsafe { asm!("msr ich_hcr_el2, {}", in(reg) ich_hcr_el2) };
}

pub fn get_ich_lr0_el2() -> u64 {
    let ich_lr0_el2: u64;
    unsafe { asm!("mrs {}, ich_lr0_el2", out(reg) ich_lr0_el2) };
    ich_lr0_el2
}

pub unsafe fn set_ich_lr0_el2(ich_lr0_el2: u64) {
    unsafe { asm!("msr ich_lr0_el2, {}", in(reg) ich_lr0_el2) };
}

pub fn get_ich_lr1_el2() -> u64 {
    let ich_lr1_el2: u64;
    unsafe { asm!("mrs {}, ich_lr1_el2", out(reg) ich_lr1_el2) };
    ich_lr1_el2
}

pub unsafe fn set_ich_lr1_el2(ich_lr1_el2: u64) {
    unsafe { asm!("msr ich_lr1_el2, {}", in(reg) ich_lr1_el2) };
}

pub fn get_ich_lr2_el2() -> u64 {
    let ich_lr2_el2: u64;
    unsafe { asm!("mrs {}, ich_lr2_el2", out(reg) ich_lr2_el2) };
    ich_lr2_el2
}

pub unsafe fn set_ich_lr2_el2(ich_lr2_el2: u64) {
    unsafe { asm!("msr ich_lr2_el2, {}", in(reg) ich_lr2_el2) };
}

pub unsafe fn set_icc_dir_el1(icc_dir_el1: u64) {
    unsafe { asm!("msr icc_dir_el1, {}", in(reg) icc_dir_el1) };
}

pub fn get_icc_ctlr_el1() -> u64 {
    let icc_ctlr_el1: u64;
    unsafe { asm!("mrs {}, icc_ctlr_el1", out(reg) icc_ctlr_el1) };
    icc_ctlr_el1
}

pub unsafe fn set_icc_ctlr_el1(icc_ctlr_el1: u64) {
    unsafe { asm!("msr icc_ctlr_el1, {}", in(reg) icc_ctlr_el1) };
}

pub unsafe fn set_cntvoff_el2(cntvoff_el2: u64) {
    unsafe { asm!("msr cntvoff_el2, {}", in(reg) cntvoff_el2) };
}

pub unsafe fn smc(mut x0: u64, x1: u64, x2: u64, x3: u64) -> u64 {
    unsafe {
        asm!("smc 0",
        inout("x0") x0,
        in("x1") x1,
        in("x2") x2,
        in("x3") x3,
        clobber_abi("C")
        )
    };
    x0
}

#[unsafe(naked)]
pub extern "C" fn core_entry() -> ! {
    naked_asm!("
            mov sp, x0
            ldp x1, x2, [x0, #-(16 * 2)] /* x1: TCR_EL2,    x2: TTBR0_EL2 */
            ldp x3, x4, [x0, #-(16 * 1)] /* x3: MAIR_EL2,   x4: SCTLR_EL2 */
            msr tcr_el2,    x1
            msr ttbr0_el2,  x2
            msr mair_el2,   x3
            isb
            msr sctlr_el2,  x4
            b   {}",
        sym crate::core_main
    )
}

pub unsafe fn get_daif_and_disable_irq_fiq() -> u64 {
    let daif: u64;
    unsafe {
        asm!("
            mrs {t},    daif
            mov {r},    {t}
            orr {t},    {t}, ( 1 << 7 /* IRQ */ ) | ( 1 << 6 /* FIQ */ )
            msr daif,   {t}
            isb",
        t = out(reg) _ ,
        r = out(reg) daif
        )
    };
    daif
}

pub unsafe fn set_daif(daif: u64) {
    unsafe {
        asm!("
            isb
            msr daif, {}",
        in(reg) daif
        )
    };
}

pub fn get_tpidr_el2() -> u64 {
    let tpidr_el2: u64;
    unsafe { asm!("mrs {}, tpidr_el2", out(reg) tpidr_el2) };
    tpidr_el2
}

pub unsafe fn set_tpidr_el2(tpidr_el2: u64) {
    unsafe { asm!("msr tpidr_el2, {}", in(reg) tpidr_el2) };
}

pub fn data_barrier() {
    unsafe { asm!("dsb sy") };
}

pub fn flush_data_cache_all() {
    let clidr: u64;
    data_barrier();
    unsafe { asm!("mrs {:x}, clidr_el1", out(reg) clidr) };
    /* Check All Cache Type */
    for cache_level in 0..7 {
        let ccsidr: u64;
        let cache_type = (clidr >> (3 * cache_level)) & 0b111;
        match cache_type {
            0b000 => {
                break; /* No Cache, Ignore the rest */
            }
            0b001 => {
                continue; /* Instruction Cache Only */
            }
            0b010..=0b100 => { /* Has data cache */ }
            _ => {
                /* Unknown Cache Type */
                continue;
            }
        }
        unsafe {
            asm!("msr csselr_el1, {:x}\nisb\nmrs {:x}, ccsidr_el1",
            in(reg) cache_level << 1,
            out(reg) ccsidr)
        };
        let num_sets = (ccsidr & ((1 << 27) - 1)) >> 13;
        let associativity = (ccsidr & ((1 << 13) - 1)) >> 3;
        let line_size = ccsidr & 0b111;
        let a = (associativity as u32).leading_zeros();
        let l = line_size + 4;
        for set in 0..=num_sets {
            for way in 0..=associativity {
                unsafe {
                    asm!("dc cisw, {:x}", in(reg) (way << a) | (set << l) | (cache_level << 1))
                };
            }
        }
    }
    data_barrier();
    unsafe { asm!("msr csselr_el1, {:x}", in(reg) 0) };
}

pub unsafe fn clean_data_cache(address: usize, size: usize) {
    let ctr_el0: u64;
    unsafe { asm!("mrs {}, ctr_el0", out(reg)ctr_el0) };
    let cache_bytes = 4;
    let cache_size = cache_bytes << ((ctr_el0 >> 16) & 0xF);
    let cache_mask = !(cache_size - 1);

    let start = address & cache_mask;
    let end = ((address + size) & cache_mask) + cache_bytes;
    for a in start..=end {
        unsafe { asm!("dc cvac, {}", in(reg) a) };
    }
    data_barrier();
}
