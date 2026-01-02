use super::*;

#[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
use simple_endian::{read_specific, Endianize};

// These are tiny helper wire structs used for reading/writing fixed regions of
// the boot sector and directory entries.
//
// Prefer fixed-size arrays when the region is just bytes; for textual regions,
// prefer the crate's fixed-text types.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(super) struct BootJumpOem {
    pub(super) jump: [u8; 3],
    pub(super) oem: [u8; 8],
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(super) struct FsTypeAscii8 {
    pub(super) bytes: [u8; 8],
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(super) struct ShortName83 {
    pub(super) name: [u8; 8],
    pub(super) ext: [u8; 3],
}

fn ascii_trim_right(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .rposition(|&b| b != b' ' && b != 0)
        .map(|i| i + 1)
        .unwrap_or(0);
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

/// BIOS Parameter Block (FAT12/16).
///
/// All multi-byte fields are little-endian on disk.
#[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Endianize)]
#[endian(le)]
#[repr(C)]
pub(super) struct BiosParameterBlock {
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

/// FAT16 extended boot record fields we care about (subset).
#[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Endianize)]
#[endian(le)]
#[repr(C)]
pub(super) struct Fat16Ebr {
    drive_number: u8,
    _reserved: u8,
    boot_sig: u8,
    volume_id: u32,
}

/// Directory entry numeric fields we care about.
///
/// This starts at offset 11 within the 32-byte entry.
#[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Endianize)]
#[endian(le)]
#[repr(C)]
pub(super) struct DirEntryRest {
    attr: u8,
    nt_reserved: u8,
    ctime_tenths: u8,
    ctime: u16,
    cdate: u16,
    adate: u16,
    first_cluster_hi: u16,
    mtime: u16,
    mdate: u16,
    first_cluster_lo: u16,
    file_size: u32,
}

#[cfg(all(feature = "derive", feature = "io-std", feature = "text_all"))]
pub fn run() {
    // --- “Driver” side: parse it back using `read_specific` ---
    let img = build_toy_fat16_image();
    let mut cur = Cursor::new(img);

// Read jump + OEM as raw bytes.
let mut jump = [0u8; 3];
let mut oem = [0u8; 8];
cur.read_exact(&mut jump).unwrap();
cur.read_exact(&mut oem).unwrap();

    // Parse BPB as endianized struct.
    let bpb: BiosParameterBlockWire = read_specific(&mut cur).expect("read BPB");

    // Read the (subset) FAT16 EBR fields we care about.
    cur.seek(SeekFrom::Start(0x24)).unwrap();
    let ebr: Fat16EbrWire = read_specific(&mut cur).expect("read FAT16 EBR");

    let bytes_per_sector = bpb.bytes_per_sector.to_native() as u64;
    let reserved = bpb.reserved_sectors.to_native() as u64;
    let fats = bpb.fats.to_native() as u64;
    let root_entries = bpb.root_entries.to_native() as usize;
    let sectors_per_fat = bpb.sectors_per_fat.to_native() as u64;

    // Boot signature at 0x1FE.
    cur.seek(SeekFrom::Start(0x1FE)).unwrap();
    let sig: u16le = read_specific(&mut cur).expect("read signature");

    // FAT16 volume label is standard ASCII (11 bytes) at 0x2B.
    let mut vol_label_raw = [0u8; 11];
    {
        let pos = cur.position();
        cur.seek(SeekFrom::Start(0x2B)).unwrap();
        cur.read_exact(&mut vol_label_raw).unwrap();
        cur.seek(SeekFrom::Start(pos)).unwrap();
    }
    let vol_label_decoded = ascii_trim_right(&vol_label_raw);

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
    println!("Volume ID: {:#X}", ebr.volume_id.to_native());
    println!("Volume label: {vol_label_decoded}");
    println!("Signature: {:#06X}", sig.to_native());

    // Compute root dir location for our toy image:
    let root_dir_sectors =
        ((root_entries * 32) as u64 + bytes_per_sector - 1) / bytes_per_sector;
    let root_dir_lba = reserved + fats * sectors_per_fat;

    // Seek to root directory and read entries, but only within the root-dir sector span.
    cur.seek(SeekFrom::Start(root_dir_lba * bytes_per_sector)).unwrap();

    let max_entries_by_sectors =
        (root_dir_sectors * bytes_per_sector) / (DIR_ENTRY_SIZE as u64);
    let max_entries = core::cmp::min(root_entries as u64, max_entries_by_sectors) as usize;

    for i in 0..max_entries {
        let mut entry_buf = [0u8; DIR_ENTRY_SIZE];
        // Our toy image might not contain a full root directory region. If we hit EOF,
        // stop gracefully instead of panicking.
        if let Err(e) = cur.read_exact(&mut entry_buf) {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                break;
            }
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

        // Parse the rest of the entry using read_specific (LE numeric fields).
        let mut rest_cur = Cursor::new(&entry_buf[11..]);
        let rest: DirEntryRestWire = match read_specific(&mut rest_cur) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let attr = rest.attr.to_native();
        // Skip long filename entries etc; keep it minimal.
        if attr == 0x0F {
            continue;
        }

        let name = dir_entry_filename(&name, &ext);
        let clus = rest.first_cluster_lo.to_native();
        let size = rest.file_size.to_native();
        println!("[{i:02}] {name:12} size={size} start_cluster={clus}");
    }

    // A tiny additional IO demo: show that the `total_sectors_*` fields are LE.
    let total: u32le = if bpb.total_sectors_16.to_native() != 0 {
        (bpb.total_sectors_16.to_native() as u32).into()
    } else {
        bpb.total_sectors_32
    };
    println!("Total sectors (interpreted): {}", total.to_native());

    println!("(root_dir_sectors computed: {root_dir_sectors})");
}
