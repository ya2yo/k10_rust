use std::mem::MaybeUninit;
use std::ptr;

use esp_idf_svc::sys::{
    esp_flash_get_size, esp_psram_get_size, esp_psram_is_initialized, heap_caps_get_free_size,
    heap_caps_get_largest_free_block, link_patches, MALLOC_CAP_INTERNAL, MALLOC_CAP_SPIRAM,
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct esp_chip_info_t {
    pub model: u32,
    pub features: u32,
    pub revision: u16,
    pub cores: u8,
}

pub const CHIP_FEATURE_WIFI_BGN: u32 = 1 << 1;
pub const CHIP_FEATURE_BLE: u32 = 1 << 4;

extern "C" {
    pub fn esp_chip_info(out_info: *mut esp_chip_info_t);
}

fn main() {
    // 必要的链接补丁与底层日志绑定
    link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    println!();
    println!("=============================================");
    println!("        K10 UNIHIKER‑S3 Hardware Test (Rust)");
    println!("=============================================");

    /* ---------- 芯片信息 ---------- */
    println!("[Chip Info]");
    let mut chip_info = MaybeUninit::<esp_chip_info_t>::uninit();
    unsafe {
        esp_chip_info(chip_info.as_mut_ptr());
    }
    let chip_info = unsafe { chip_info.assume_init() };

    let has_wifi = (chip_info.features & CHIP_FEATURE_WIFI_BGN) != 0;
    let has_ble = (chip_info.features & CHIP_FEATURE_BLE) != 0;

    println!("  CPU cores        : {}", chip_info.cores);
    println!("  Chip revision    : {}", chip_info.revision);
    println!("  Features mask    : 0x{:08X}", chip_info.features);
    println!("  WiFi             : {}", if has_wifi { "YES" } else { "NO" });
    println!("  BLE              : {}", if has_ble { "YES" } else { "NO" });

    /* ---------- Flash 信息 ---------- */
    println!("\n[Flash Info]");
    let mut flash_size: u32 = 0;
    unsafe {
        // 传入 NULL 默认检测主启动 Flash
        esp_flash_get_size(ptr::null_mut(), &mut flash_size as *mut u32);
    }
    let flash_mb = (flash_size as f64) / (1024.0 * 1024.0);
    println!("  Flash total size : {:.2} MB ({} bytes)", flash_mb, flash_size);

    /* ---------- PSRAM 信息 ---------- */
    println!("\n[PSRAM (Octal‑OPI) Info]");
    let psram_ok = unsafe { esp_psram_is_initialized() };
    if psram_ok {
        let psram_total = unsafe { esp_psram_get_size() };
        let psram_free = unsafe { heap_caps_get_free_size(MALLOC_CAP_SPIRAM) };
        let psram_largest_blk = unsafe { heap_caps_get_largest_free_block(MALLOC_CAP_SPIRAM) };

        let psram_mb = (psram_total as f64) / (1024.0 * 1024.0);
        println!("  PSRAM init ok    : YES");
        println!("  PSRAM total      : {:.2} MB ({} bytes)", psram_mb, psram_total);
        println!("  PSRAM free heap  : {} bytes", psram_free);
        println!("  PSRAM max block  : {} bytes", psram_largest_blk);
    } else {
        println!("  PSRAM init ok    : ❌ FAILED! Check sdkconfig PSRAM settings");
    }

    /* ---------- 内部 SRAM (On-Chip RAM) ---------- */
    println!("\n[On‑Chip Internal SRAM (not PSRAM)]");
    let int_ram_free = unsafe { heap_caps_get_free_size(MALLOC_CAP_INTERNAL) };
    let int_ram_largest = unsafe { heap_caps_get_largest_free_block(MALLOC_CAP_INTERNAL) };
    println!("  Internal free    : {} bytes", int_ram_free);
    println!("  Internal max blk : {} bytes", int_ram_largest);

    println!("\n=============================================\n");
}
