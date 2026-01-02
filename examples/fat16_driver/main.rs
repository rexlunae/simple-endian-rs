//! A tiny FAT16 “driver” example.
//!
//! This demonstrates:
//! - using `io-std` helpers (`read_specific`) to parse **little-endian on-disk structs**
//! - using the `derive` feature (`#[derive(Endianize)]`) to generate `*Wire` types
//! - using fixed-size UTF-16LE text fields (volume label) from the text features
//!
//! It is intentionally minimal (not a complete FAT implementation).
//!
//! Run:
//!
//! ```sh
//! cargo run --example fat16_driver --features "derive io-std text_all"
//! ```

use simple_endian::{read_specific, u16le, u32le, FixedUtf16LeSpacePadded};
use simple_endian_derive::Endianize;
use std::io::{Cursor, Read, Seek, SeekFrom};

const BYTES_PER_SECTOR: usize = 512;
const DIR_ENTRY_SIZE: usize = 32;

/// BIOS Parameter Block (FAT12/16). All multi-byte fields are little-endian.
#[derive(Debug, Clone, Copy, Endianize)]
#[endian(le)]
#[repr(C)]
struct BiosParameterBlock {
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fats: u8,
    root_entries: u16,
    total_sectors_16: u16,
    media: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    heads: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
}

/// FAT16 “extended boot record” (subset).
#[derive(Debug, Clone, Copy, Endianize)]
#[endian(le)]
#[repr(C)]
struct Fat16Extended {
    drive_number: u8,
    _reserved: u8,
    boot_sig: u8,
    volume_id: u32,
}

fn ascii_trim_right(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .rposition(|&b| b != b' ' && b != 0)
        .map(|i| i + 1)
        .unwrap_or(0);
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

fn dir_entry_filename(name: &[u8; 8], ext: &[u8; 3]) -> String {
    let name = ascii_trim_right(name);
    let ext = ascii_trim_right(ext);
    if ext.is_empty() {
        name
    } else {
        format!("{name}.{ext}")
    }
}

fn main() {
    // Build a tiny in-memory “disk image”:
    // - sector 0: boot sector (512 bytes)
    // - sector 1: (dummy) FAT
    // - sector 2: root directory (we place one entry)
    let bytes_per_sector = BYTES_PER_SECTOR;
    let mut img = vec![0u8; 3 * bytes_per_sector];

    // --- Write boot-sector pieces into the 512-byte sector ---
    // Layout (classic):
    // 0x00: jump (3)
    // 0x03: OEM (8)
    // 0x0B: BPB (25)
    // 0x24: FAT16 extended (26) -- in the classic FAT16 boot sector this is the EBR region
    // 0x1FE: signature 0x55AA

    img[0..3].copy_from_slice(&[0xEB, 0x3C, 0x90]);
    img[3..11].copy_from_slice(b"MSDOS5.0");

    // Write BPB explicitly (avoids any struct-layout/padding surprises).
    // Offsets are from start of boot sector.
    img[0x0B..0x0D].copy_from_slice(&(bytes_per_sector as u16).to_le_bytes()); // bytes/sector
    img[0x0D] = 1; // sectors/cluster
    img[0x0E..0x10].copy_from_slice(&1u16.to_le_bytes()); // reserved
    img[0x10] = 1; // fats
    img[0x11..0x13].copy_from_slice(&16u16.to_le_bytes()); // root entries
    img[0x13..0x15].copy_from_slice(&0u16.to_le_bytes()); // total sectors 16
    img[0x15] = 0xF8; // media
    img[0x16..0x18].copy_from_slice(&1u16.to_le_bytes()); // sectors/FAT
    img[0x18..0x1A].copy_from_slice(&32u16.to_le_bytes()); // sectors/track
    img[0x1A..0x1C].copy_from_slice(&64u16.to_le_bytes()); // heads
    img[0x1C..0x20].copy_from_slice(&0u32.to_le_bytes()); // hidden sectors
    img[0x20..0x24].copy_from_slice(&3u32.to_le_bytes()); // total sectors 32

    // FAT16 EBR fields.
    img[0x24] = 0x80; // drive number
    img[0x25] = 0x00; // reserved
    img[0x26] = 0x29; // boot sig
    img[0x27..0x2B].copy_from_slice(&0x1234_5678u32.to_le_bytes()); // volume id

    // Volume label is 11 bytes of ASCII at 0x2B in the FAT16 EBR.
    // We'll decode it as UTF-16LE (fixed + padded) later to exercise text support.
    img[0x2B..0x36].copy_from_slice(b"VOL_LABEL  ");

    // fs_type is raw bytes in the boot sector; keep it raw bytes.
    // NOTE: must be written *after* the EBR chunk; it overlaps that region.
    img[0x36..0x3E].copy_from_slice(b"FAT16   ");

    img[0x1FE] = 0x55;
    img[0x1FF] = 0xAA;

    // Root directory entry in sector 2.
    // Root directory entry bytes:
    // - name(8) + ext(3) are raw bytes
    // - the rest is the endianized scalar struct
    let name_bytes = *b"HELLO   ";
    let ext_bytes = *b"TXT";

    let root_sector_off = 2 * bytes_per_sector;
    img[root_sector_off..root_sector_off + 8].copy_from_slice(&name_bytes);
    img[root_sector_off + 8..root_sector_off + 11].copy_from_slice(&ext_bytes);

    // Write the remaining 21 bytes of the entry explicitly (all little-endian fields).
    // Offsets are from the start of the 32-byte directory entry.
    let eoff = root_sector_off;
    img[eoff + 11] = 0x20; // attr: archive
    img[eoff + 12] = 0x00; // nt reserved
    img[eoff + 13] = 0x00; // ctime tenths
    img[eoff + 14..eoff + 16].copy_from_slice(&0u16.to_le_bytes()); // ctime
    img[eoff + 16..eoff + 18].copy_from_slice(&0u16.to_le_bytes()); // cdate
    img[eoff + 18..eoff + 20].copy_from_slice(&0u16.to_le_bytes()); // adate
    img[eoff + 20..eoff + 22].copy_from_slice(&0u16.to_le_bytes()); // cluster hi
    img[eoff + 22..eoff + 24].copy_from_slice(&0u16.to_le_bytes()); // mtime
    img[eoff + 24..eoff + 26].copy_from_slice(&0u16.to_le_bytes()); // mdate
    img[eoff + 26..eoff + 28].copy_from_slice(&2u16.to_le_bytes()); // cluster lo
    img[eoff + 28..eoff + 32].copy_from_slice(&13u32.to_le_bytes()); // size

    // Any remaining bytes in the 32-byte dir entry are already zeroed in `img`.
    // We wrote the first 11 bytes (name+ext) already; now write the 21-byte remainder.
    // Any remaining bytes in the 32-byte dir entry are already zeroed in `img`.

    // --- “Driver” side: parse it back using `read_specific` ---

    let mut cur = Cursor::new(img);

    // Read jump + OEM as raw bytes.
    let mut jump = [0u8; 3];
    let mut oem = [0u8; 8];
    cur.read_exact(&mut jump).unwrap();
    cur.read_exact(&mut oem).unwrap();

    // Parse BPB as endianized struct.
    let bpb_read: BiosParameterBlockWire = read_specific(&mut cur).expect("read BPB");

    // Seek to FAT16 EBR region and parse.
    // For this example, `Fat16ExtendedWire` starts at the EBR “drive number” offset (0x24),
    // so it includes: drive/reserved/bootsig/volume_id/volume_label.
    cur.seek(SeekFrom::Start(0x24)).unwrap();
    let ext_read: Fat16ExtendedWire = read_specific(&mut cur).expect("read FAT16 ext");

    // Boot signature at 0x1FE.
    cur.seek(SeekFrom::Start(0x1FE)).unwrap();
    let sig: u16le = read_specific(&mut cur).expect("read signature");

    let bytes_per_sector = bpb_read.bytes_per_sector.to_native() as u64;
    let root_entries = bpb_read.root_entries.to_native() as usize;

    // Read the raw volume label bytes and interpret them as fixed UTF-16LE just as a demo.
    // Note: a real FAT16 label is usually ASCII, not UTF-16.
    let mut vol_label_raw = [0u8; 11];
    {
        let pos = cur.position();
        cur.seek(SeekFrom::Start(0x2B)).unwrap();
        cur.read_exact(&mut vol_label_raw).unwrap();
        cur.seek(SeekFrom::Start(pos)).unwrap();
    }
    let vol_label_utf16: FixedUtf16LeSpacePadded<11> = String::from_utf8_lossy(&vol_label_raw)
        .as_ref()
        .try_into()
        .expect("fits");
    let vol_label_decoded = String::try_from(&vol_label_utf16).expect("utf16 label");

    println!("Jump: {:02X?}", jump);
    println!("OEM: {}", String::from_utf8_lossy(&oem));
    println!("FAT16 boot: bytes/sector={bytes_per_sector}, root_entries={root_entries}");
    let mut fs_type = [0u8; 8];
    {
        // fs_type is at 0x36 in the classic FAT16 layout
        let pos = cur.position();
        cur.seek(SeekFrom::Start(0x36)).unwrap();
        cur.read_exact(&mut fs_type).unwrap();
        cur.seek(SeekFrom::Start(pos)).unwrap();
    }
    println!("FS type: {}", String::from_utf8_lossy(&fs_type));
    println!("Volume ID: {:#X}", ext_read.volume_id.to_native());
    println!("Volume label (UTF-16LE fixed): {vol_label_decoded}");
    println!("Signature: {:#06X}", sig.to_native());

    // Compute root dir location for our toy image:
    let fats = bpb_read.fats.to_native() as u64;
    let reserved = bpb_read.reserved_sectors.to_native() as u64;
    let sectors_per_fat = bpb_read.sectors_per_fat.to_native() as u64;

    let root_dir_sectors = ((root_entries * 32) as u64 + bytes_per_sector - 1) / bytes_per_sector;
    let root_dir_lba = reserved + fats * sectors_per_fat;

    // Seek to root directory and read entries, but only within the root-dir sector span.
    cur.seek(SeekFrom::Start(root_dir_lba * bytes_per_sector)).unwrap();

    let max_entries_by_sectors = (root_dir_sectors * bytes_per_sector) / (DIR_ENTRY_SIZE as u64);
    let max_entries = core::cmp::min(root_entries as u64, max_entries_by_sectors) as usize;

    for i in 0..max_entries {
        let mut entry_buf = [0u8; DIR_ENTRY_SIZE];
        if let Err(e) = cur.read_exact(&mut entry_buf) {
            panic!("read dir entry: {e}");
        }

        if entry_buf[0] == 0x00 {
            break; // end of directory
        }
        if entry_buf[0] == 0xE5 {
            continue; // deleted
        }

        let name = <[u8; 8]>::try_from(&entry_buf[0..8]).unwrap();
        let ext = <[u8; 3]>::try_from(&entry_buf[8..11]).unwrap();

        let attr = entry_buf[11];
        // Skip long filename entries etc; keep it minimal.
        if attr == 0x0F {
            continue;
        }

        let name = dir_entry_filename(&name, &ext);
        let clus = u16::from_le_bytes([entry_buf[26], entry_buf[27]]);
        let size = u32::from_le_bytes([entry_buf[28], entry_buf[29], entry_buf[30], entry_buf[31]]);
        println!("[{i:02}] {name:12} size={size} start_cluster={clus}");
    }

    // A tiny additional IO demo: show that the `total_sectors_*` fields are LE.
    let total: u32le = if bpb_read.total_sectors_16.to_native() != 0 {
        (bpb_read.total_sectors_16.to_native() as u32).into()
    } else {
        bpb_read.total_sectors_32
    };
    println!("Total sectors (interpreted): {}", total.to_native());

    println!("(root_dir_sectors computed: {root_dir_sectors})");
}
