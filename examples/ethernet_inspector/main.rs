//! Example: Ethernet packet inspector (Ethernet II + common L3/L4 protocols)
//!
//! This example demonstrates:
//! - using `#[derive(Endianize)]` to define on-wire headers
//! - using `read_specific` / `write_specific` (`io-std`) to read/write headers
//! - parsing common network traffic: VLAN, ARP, IPv4, IPv6, TCP, UDP, ICMP
//!
//! The input format is a simple length-prefixed stream:
//! - repeated records of: `u16be length` + `length` bytes of frame
//!
//! There is also a basic PCAP reader mode (classic PCAP, not pcapng):
//!
//! ```sh
//! cargo run --example ethernet_inspector --features "derive io-std" -- --pcap capture.pcap
//! ```
//!
//! That keeps the example self-contained (no pcap dependency) while still being
//! easy to generate from a capture tool.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example ethernet_inspector --features "derive io-std" -- <file>
//! ```
//!
//! Or read from stdin:
//!
//! ```sh
//! cat frames.bin | cargo run --example ethernet_inspector --features "derive io-std"
//! ```
//!
//! You can also generate a few mock frames in-process and immediately inspect them:
//!
//! ```sh
//! cargo run --example ethernet_inspector --features "derive io-std" -- --demo
//! ```
//!
//! Or generate a Wireshark-friendly classic PCAP (Ethernet linktype) from the same mock frames:
//!
//! ```sh
//! cargo run --example ethernet_inspector --features "derive io-std" -- --demo-pcap /tmp/demo.pcap
//! ```
//!
//! Sample output:
//!
//! ```text
//! 0000: ETH 02:00:00:00:00:01 -> ff:ff:ff:ff:ff:ff IPv4 UDP 192.168.0.2:5353 -> 224.0.0.251:5353 (mDNS)
//! 0001: ETH 02:00:00:00:00:02 -> ff:ff:ff:ff:ff:ff ARP request 192.168.0.10(02:00:00:00:00:02) -> 192.168.0.1(00:00:00:00:00:00)
//! 0002: ETH 02:00:00:00:00:03 -> 10:20:30:40:50:60 IPv4 TCP 10.0.0.2:51515 -> 93.184.216.34:80 flags=SYN (HTTP)
//! 0003: ETH 02:00:00:00:00:04 -> 10:11:12:13:14:15 IPv6 TCP 2001:db8:0:0:0:0:0:1:51516 -> 2001:db8:0:0:0:0:0:2:443 flags=SYN (HTTPS)
//! 0004: ETH 02:00:00:00:00:05 -> aa:bb:cc:dd:ee:ff vlan=42 IPv4 UDP 192.168.42.10:53000 -> 192.168.42.1:53 (DNS)
//! ```

#![cfg_attr(
    not(all(feature = "derive", feature = "io-std")),
    allow(dead_code, unused_imports)
)]

#[cfg(all(feature = "derive", feature = "io-std"))]
mod demo {
    use simple_endian::{Endianize, read_specific, u16be, write_specific};
    use std::io::{self, Read, Write};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum EtherType {
        Ipv4,
        Arp,
        Ipv6,
        Vlan,
        Other(u16),
    }

    impl From<u16> for EtherType {
        fn from(v: u16) -> Self {
            match v {
                0x0800 => EtherType::Ipv4,
                0x0806 => EtherType::Arp,
                0x86DD => EtherType::Ipv6,
                0x8100 => EtherType::Vlan,
                _ => EtherType::Other(v),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum IpProto {
        Icmp,
        Tcp,
        Udp,
        Icmpv6,
        Other(u8),
    }

    impl From<u8> for IpProto {
        fn from(v: u8) -> Self {
            match v {
                1 => IpProto::Icmp,
                6 => IpProto::Tcp,
                17 => IpProto::Udp,
                58 => IpProto::Icmpv6,
                _ => IpProto::Other(v),
            }
        }
    }

    fn mac_to_string(mac: &[u8; 6]) -> String {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        )
    }

    fn ipv4_to_string(ip: &[u8; 4]) -> String {
        format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
    }

    fn ipv6_to_string(ip: &[u8; 16]) -> String {
        // Minimal formatter: group into 8 u16 hextets.
        let mut parts = [0u16; 8];
        for i in 0..8 {
            parts[i] = u16::from_be_bytes([ip[2 * i], ip[2 * i + 1]]);
        }
        format!(
            "{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            parts[0], parts[1], parts[2], parts[3], parts[4], parts[5], parts[6], parts[7]
        )
    }

    fn tcp_flags_to_string(raw_flags: u16) -> String {
        // TCP flags are the low 9 bits of the 16-bit field that also includes data offset.
        // Bits: NS (bit 8), CWR (7), ECE (6), URG (5), ACK (4), PSH (3), RST (2), SYN (1), FIN (0)
        let mut out = String::new();
        let mut push = |s: &str, first: &mut bool| {
            if !*first {
                out.push('|');
            }
            out.push_str(s);
            *first = false;
        };
        let mut first = true;
        if (raw_flags & 0x0100) != 0 {
            push("NS", &mut first);
        }
        if (raw_flags & 0x0080) != 0 {
            push("CWR", &mut first);
        }
        if (raw_flags & 0x0040) != 0 {
            push("ECE", &mut first);
        }
        if (raw_flags & 0x0020) != 0 {
            push("URG", &mut first);
        }
        if (raw_flags & 0x0010) != 0 {
            push("ACK", &mut first);
        }
        if (raw_flags & 0x0008) != 0 {
            push("PSH", &mut first);
        }
        if (raw_flags & 0x0004) != 0 {
            push("RST", &mut first);
        }
        if (raw_flags & 0x0002) != 0 {
            push("SYN", &mut first);
        }
        if (raw_flags & 0x0001) != 0 {
            push("FIN", &mut first);
        }
        if out.is_empty() {
            out.push('-');
        }
        out
    }

    #[derive(Endianize, Debug, Clone, Copy)]
    #[endian(be)]
    #[repr(C)]
    struct Ethernet2Header {
        dst: [u8; 6],
        src: [u8; 6],
        ethertype: u16,
    }

    #[derive(Endianize, Debug, Clone, Copy)]
    #[endian(be)]
    #[repr(C)]
    struct VlanTag {
        // 3 bits PCP, 1 bit DEI, 12 bits VLAN ID
        tci: u16,
        // inner EtherType
        ethertype: u16,
    }

    #[derive(Endianize, Debug, Clone, Copy)]
    #[endian(be)]
    #[repr(C)]
    struct ArpHeader {
        htype: u16,
        ptype: u16,
        hlen: u8,
        plen: u8,
        oper: u16,
        sender_hw: [u8; 6],
        sender_ip: [u8; 4],
        target_hw: [u8; 6],
        target_ip: [u8; 4],
    }

    #[derive(Endianize, Debug, Clone, Copy)]
    #[endian(be)]
    #[repr(C)]
    struct Ipv4Header {
        version_ihl: u8,
        dscp_ecn: u8,
        total_len: u16,
        ident: u16,
        flags_frag: u16,
        ttl: u8,
        protocol: u8,
        header_checksum: u16,
        src: [u8; 4],
        dst: [u8; 4],
    }

    #[derive(Endianize, Debug, Clone, Copy)]
    #[endian(be)]
    #[repr(C)]
    struct Ipv6Header {
        ver_tc_flow: u32,
        payload_len: u16,
        next_header: u8,
        hop_limit: u8,
        src: [u8; 16],
        dst: [u8; 16],
    }

    #[derive(Endianize, Debug, Clone, Copy)]
    #[endian(be)]
    #[repr(C)]
    struct UdpHeader {
        src_port: u16,
        dst_port: u16,
        len: u16,
        checksum: u16,
    }

    #[derive(Endianize, Debug, Clone, Copy)]
    #[endian(be)]
    #[repr(C)]
    struct TcpHeader {
        src_port: u16,
        dst_port: u16,
        seq: u32,
        ack: u32,
        data_offset_reserved_flags: u16,
        window: u16,
        checksum: u16,
        urgent: u16,
    }

    #[derive(Endianize, Debug, Clone, Copy)]
    #[endian(be)]
    #[repr(C)]
    struct IcmpHeader {
        type_: u8,
        code: u8,
        checksum: u16,
        rest: u32,
    }

    fn tcp_header_len_bytes(h: &TcpHeaderWire) -> usize {
        // High 4 bits of the first byte in the u16 are data offset in 32-bit words.
        let raw = h.data_offset_reserved_flags.to_native();
        let offset_words = ((raw >> 12) & 0xF) as usize;
        offset_words * 4
    }

    fn parse_ipv6_next_header(mut next: u8, cur: &mut io::Cursor<&[u8]>) -> IpProto {
        // Minimal extension header walking to reach upper-layer protocol.
        // We don't fully expose extension metadata; we just skip them.
        //
        // Supported extensions: Hop-by-Hop (0), Routing (43), Fragment (44), Destination Options (60).
        loop {
            match next {
                0 | 43 | 60 => {
                    // Generic options header: next_header (u8), hdr_ext_len (u8) in 8-octet units minus 1.
                    let mut hdr = [0u8; 2];
                    if cur.read_exact(&mut hdr).is_err() {
                        return IpProto::Other(next);
                    }
                    let nh = hdr[0];
                    let hdr_ext_len = hdr[1] as usize;
                    let bytes = (hdr_ext_len + 1) * 8;
                    // We've already consumed 2 bytes.
                    if bytes < 2 {
                        return IpProto::Other(next);
                    }
                    let mut skip = vec![0u8; bytes - 2];
                    if cur.read_exact(&mut skip).is_err() {
                        return IpProto::Other(next);
                    }
                    next = nh;
                }
                44 => {
                    // Fragment header is fixed 8 bytes: next_header (1), reserved (1), fragment (2), ident (4)
                    let mut hdr = [0u8; 8];
                    if cur.read_exact(&mut hdr).is_err() {
                        return IpProto::Other(next);
                    }
                    next = hdr[0];
                }
                _ => return IpProto::from(next),
            }
        }
    }

    fn ipv4_header_len_bytes(h: &Ipv4HeaderWire) -> usize {
        let ihl_words = (h.version_ihl.to_native() & 0x0F) as usize;
        ihl_words * 4
    }

    fn guess_service(proto: IpProto, src_port: u16, dst_port: u16) -> Option<&'static str> {
        let p = src_port.min(dst_port);
        match (proto, p) {
            (IpProto::Udp, 53) => Some("DNS"),
            (IpProto::Tcp, 53) => Some("DNS"),
            (IpProto::Udp, 67) => Some("DHCP"),
            (IpProto::Udp, 68) => Some("DHCP"),
            (IpProto::Udp, 123) => Some("NTP"),
            (IpProto::Udp, 5353) => Some("mDNS"),
            (IpProto::Udp, 1900) => Some("SSDP"),
            (IpProto::Tcp, 80) => Some("HTTP"),
            (IpProto::Tcp, 443) => Some("HTTPS"),
            (IpProto::Tcp, 22) => Some("SSH"),
            (IpProto::Tcp, 25) => Some("SMTP"),
            (IpProto::Tcp, 110) => Some("POP3"),
            (IpProto::Tcp, 143) => Some("IMAP"),
            (IpProto::Tcp, 1883) => Some("MQTT"),
            _ => None,
        }
    }

    fn parse_eth_frame(frame: &[u8]) -> String {
        let mut cur = io::Cursor::new(frame);
        let eth: Ethernet2HeaderWire = match read_specific(&mut cur) {
            Ok(v) => v,
            Err(e) => return format!("ETH <short frame: {e}>"),
        };

        let mut summary = String::new();
        let src_mac = mac_to_string(&eth.src);
        let dst_mac = mac_to_string(&eth.dst);

        let mut ethertype = EtherType::from(eth.ethertype.to_native());
        let mut vlan: Option<u16> = None;

        // Handle 802.1Q VLAN tag (single tag).
        if ethertype == EtherType::Vlan {
            let tag: VlanTagWire = match read_specific(&mut cur) {
                Ok(v) => v,
                Err(e) => {
                    return format!("ETH {src_mac} -> {dst_mac} VLAN <short tag: {e}>",);
                }
            };
            let tci = tag.tci.to_native();
            vlan = Some(tci & 0x0FFF);
            ethertype = EtherType::from(tag.ethertype.to_native());
        }

        summary.push_str(&format!("ETH {src_mac} -> {dst_mac}"));
        if let Some(v) = vlan {
            summary.push_str(&format!(" vlan={v}"));
        }

        match ethertype {
            EtherType::Arp => {
                let arp: ArpHeaderWire = match read_specific(&mut cur) {
                    Ok(v) => v,
                    Err(e) => return format!("{summary} ARP <short: {e}>"),
                };
                let oper = arp.oper.to_native();
                let op = match oper {
                    1 => "request",
                    2 => "reply",
                    _ => "other",
                };
                summary.push_str(&format!(
                    " ARP {op} {}({}) -> {}({})",
                    ipv4_to_string(&arp.sender_ip),
                    mac_to_string(&arp.sender_hw),
                    ipv4_to_string(&arp.target_ip),
                    mac_to_string(&arp.target_hw)
                ));
                summary
            }
            EtherType::Ipv4 => {
                let ip: Ipv4HeaderWire = match read_specific(&mut cur) {
                    Ok(v) => v,
                    Err(e) => return format!("{summary} IPv4 <short: {e}>"),
                };
                let ihl = ipv4_header_len_bytes(&ip);
                if ihl < 20 {
                    return format!("{summary} IPv4 <bad ihl={ihl}>");
                }
                let src = ipv4_to_string(&ip.src);
                let dst = ipv4_to_string(&ip.dst);
                let proto = IpProto::from(ip.protocol.to_native());

                // Skip IPv4 options if present.
                let already = 20usize;
                if ihl > already {
                    let mut skip = vec![0u8; ihl - already];
                    if cur.read_exact(&mut skip).is_err() {
                        return format!("{summary} IPv4 <short options>");
                    }
                }

                match proto {
                    IpProto::Udp => {
                        let udp: UdpHeaderWire = match read_specific(&mut cur) {
                            Ok(v) => v,
                            Err(e) => {
                                return format!("{summary} IPv4 UDP {src} -> {dst} <short: {e}>");
                            }
                        };
                        let sp = udp.src_port.to_native();
                        let dp = udp.dst_port.to_native();
                        summary.push_str(&format!(" IPv4 UDP {src}:{sp} -> {dst}:{dp}"));
                        if let Some(svc) = guess_service(proto, sp, dp) {
                            summary.push_str(&format!(" ({svc})"));
                        }
                        summary
                    }
                    IpProto::Tcp => {
                        let tcp: TcpHeaderWire = match read_specific(&mut cur) {
                            Ok(v) => v,
                            Err(e) => {
                                return format!("{summary} IPv4 TCP {src} -> {dst} <short: {e}>");
                            }
                        };
                        let sp = tcp.src_port.to_native();
                        let dp = tcp.dst_port.to_native();
                        let hlen = tcp_header_len_bytes(&tcp);
                        if hlen < 20 {
                            return format!(
                                "{summary} IPv4 TCP {src}:{sp} -> {dst}:{dp} <bad hlen={hlen}>"
                            );
                        }
                        // Skip any TCP options.
                        if hlen > 20 {
                            let mut skip = vec![0u8; hlen - 20];
                            if cur.read_exact(&mut skip).is_err() {
                                return format!(
                                    "{summary} IPv4 TCP {src}:{sp} -> {dst}:{dp} <short options>"
                                );
                            }
                        }
                        let flags = tcp_flags_to_string(
                            tcp.data_offset_reserved_flags.to_native() & 0x01FF,
                        );
                        summary
                            .push_str(&format!(" IPv4 TCP {src}:{sp} -> {dst}:{dp} flags={flags}"));
                        if let Some(svc) = guess_service(proto, sp, dp) {
                            summary.push_str(&format!(" ({svc})"));
                        }
                        summary
                    }
                    IpProto::Icmp => {
                        let icmp: IcmpHeaderWire = match read_specific(&mut cur) {
                            Ok(v) => v,
                            Err(e) => {
                                return format!("{summary} IPv4 ICMP {src} -> {dst} <short: {e}>");
                            }
                        };
                        summary.push_str(&format!(
                            " IPv4 ICMP {src} -> {dst} type={} code={}",
                            icmp.type_.to_native(),
                            icmp.code.to_native()
                        ));
                        summary
                    }
                    IpProto::Other(n) => format!("{summary} IPv4 {src} -> {dst} proto={n}"),
                    IpProto::Icmpv6 => {
                        format!("{summary} IPv4 {src} -> {dst} (bad: icmpv6 in ipv4?)")
                    }
                }
            }
            EtherType::Ipv6 => {
                let ip: Ipv6HeaderWire = match read_specific(&mut cur) {
                    Ok(v) => v,
                    Err(e) => return format!("{summary} IPv6 <short: {e}>"),
                };
                let src = ipv6_to_string(&ip.src);
                let dst = ipv6_to_string(&ip.dst);
                let proto = parse_ipv6_next_header(ip.next_header.to_native(), &mut cur);

                match proto {
                    IpProto::Udp => {
                        let udp: UdpHeaderWire = match read_specific(&mut cur) {
                            Ok(v) => v,
                            Err(e) => {
                                return format!("{summary} IPv6 UDP {src} -> {dst} <short: {e}>");
                            }
                        };
                        let sp = udp.src_port.to_native();
                        let dp = udp.dst_port.to_native();
                        summary.push_str(&format!(" IPv6 UDP {src}:{sp} -> {dst}:{dp}"));
                        if let Some(svc) = guess_service(proto, sp, dp) {
                            summary.push_str(&format!(" ({svc})"));
                        }
                        summary
                    }
                    IpProto::Tcp => {
                        let tcp: TcpHeaderWire = match read_specific(&mut cur) {
                            Ok(v) => v,
                            Err(e) => {
                                return format!("{summary} IPv6 TCP {src} -> {dst} <short: {e}>");
                            }
                        };
                        let sp = tcp.src_port.to_native();
                        let dp = tcp.dst_port.to_native();
                        let hlen = tcp_header_len_bytes(&tcp);
                        if hlen < 20 {
                            return format!(
                                "{summary} IPv6 TCP {src}:{sp} -> {dst}:{dp} <bad hlen={hlen}>"
                            );
                        }
                        if hlen > 20 {
                            let mut skip = vec![0u8; hlen - 20];
                            if cur.read_exact(&mut skip).is_err() {
                                return format!(
                                    "{summary} IPv6 TCP {src}:{sp} -> {dst}:{dp} <short options>"
                                );
                            }
                        }
                        let flags = tcp_flags_to_string(
                            tcp.data_offset_reserved_flags.to_native() & 0x01FF,
                        );
                        summary
                            .push_str(&format!(" IPv6 TCP {src}:{sp} -> {dst}:{dp} flags={flags}"));
                        if let Some(svc) = guess_service(proto, sp, dp) {
                            summary.push_str(&format!(" ({svc})"));
                        }
                        summary
                    }
                    IpProto::Icmpv6 => {
                        let icmp: IcmpHeaderWire = match read_specific(&mut cur) {
                            Ok(v) => v,
                            Err(e) => {
                                return format!(
                                    "{summary} IPv6 ICMPv6 {src} -> {dst} <short: {e}>"
                                );
                            }
                        };
                        summary.push_str(&format!(
                            " IPv6 ICMPv6 {src} -> {dst} type={} code={}",
                            icmp.type_.to_native(),
                            icmp.code.to_native(),
                        ));
                        summary
                    }
                    IpProto::Other(n) => format!("{summary} IPv6 {src} -> {dst} next={n}"),
                    IpProto::Icmp => format!("{summary} IPv6 {src} -> {dst} (bad: icmp in ipv6?)"),
                }
            }
            EtherType::Other(n) => format!("{summary} ethertype=0x{n:04x}"),
            EtherType::Vlan => format!("{summary} VLAN (unexpected nested?)"),
        }
    }

    #[derive(Endianize, Debug, Clone, Copy)]
    #[endian(le)]
    #[repr(C)]
    struct PcapGlobalHeader {
        magic: u32,
        version_major: u16,
        version_minor: u16,
        thiszone: u32,
        sigfigs: u32,
        snaplen: u32,
        network: u32,
    }

    #[derive(Endianize, Debug, Clone, Copy)]
    #[endian(le)]
    #[repr(C)]
    struct PcapRecordHeader {
        ts_sec: u32,
        ts_usec: u32,
        incl_len: u32,
        orig_len: u32,
    }

    /// Read a length-prefixed frame stream: (u16be len, len bytes frame) repeated.
    fn read_frames(mut input: impl Read) -> io::Result<Vec<Vec<u8>>> {
        let mut frames = Vec::new();
        loop {
            let len: u16be = match read_specific(&mut input) {
                Ok(v) => v,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };
            let mut buf = vec![0u8; len.to_native() as usize];
            input.read_exact(&mut buf)?;
            frames.push(buf);
        }
        Ok(frames)
    }

    fn read_pcap(mut input: impl Read) -> io::Result<Vec<Vec<u8>>> {
        // Classic pcap reader (little-endian header).
        let hdr: PcapGlobalHeaderWire = read_specific(&mut input)?;
        let magic = hdr.magic.to_native();
        let _ns_resolution = match magic {
            0xd4c3b2a1 => false,
            0x4d3cb2a1 => true,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unsupported pcap magic 0x{magic:08x} (expected LE)"),
                ));
            }
        };

        let network = hdr.network.to_native();

        // DLT_EN10MB (Ethernet) == 1.
        if network != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported pcap network/linktype {network} (expected 1 = Ethernet)"),
            ));
        }

        let mut frames = Vec::new();
        loop {
            let rec: PcapRecordHeaderWire = match read_specific(&mut input) {
                Ok(v) => v,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };
            let _ = (
                rec.ts_sec.to_native(),
                rec.ts_usec.to_native(),
                rec.orig_len.to_native(),
            );
            let incl_len = rec.incl_len.to_native() as usize;
            let mut buf = vec![0u8; incl_len];
            input.read_exact(&mut buf)?;
            frames.push(buf);
        }
        Ok(frames)
    }

    fn write_frames(mut out: impl Write, frames: &[Vec<u8>]) -> io::Result<()> {
        for f in frames {
            if f.len() > u16::MAX as usize {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "frame too large",
                ));
            }
            let len: u16be = (f.len() as u16).into();
            write_specific(&mut out, &len)?;
            out.write_all(f)?;
        }
        Ok(())
    }

    fn write_pcap(mut out: impl Write, frames: &[Vec<u8>]) -> io::Result<()> {
        // Minimal classic PCAP writer (little-endian) using crate IO.
        let gh = PcapGlobalHeaderWire {
            magic: 0xd4c3b2a1u32.into(),
            version_major: 2u16.into(),
            version_minor: 4u16.into(),
            thiszone: 0u32.into(),
            sigfigs: 0u32.into(),
            snaplen: 65535u32.into(),
            network: 1u32.into(),
        };
        write_specific(&mut out, &gh)?;

        // Record headers + packet bytes.
        for f in frames {
            let incl = f.len();
            if incl > u32::MAX as usize {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "frame too large",
                ));
            }
            let rh = PcapRecordHeaderWire {
                // timestamp (0,0) for demo; Wireshark is fine with this.
                ts_sec: 0u32.into(),
                ts_usec: 0u32.into(),
                incl_len: (incl as u32).into(),
                orig_len: (incl as u32).into(),
            };
            write_specific(&mut out, &rh)?;
            out.write_all(f)?;
        }
        Ok(())
    }

    fn make_demo_frames() -> Vec<Vec<u8>> {
        let mut frames = Vec::new();

        // 0) IPv4 UDP mDNS (broadcast Ethernet, multicast IP).
        {
            let eth = Ethernet2HeaderWire {
                dst: [0xff; 6],
                src: [0x02, 0x00, 0x00, 0x00, 0x00, 0x01],
                ethertype: 0x0800u16.into(),
            };
            let ip = Ipv4HeaderWire {
                version_ihl: 0x45u8.into(),
                dscp_ecn: 0u8.into(),
                total_len: (20u16 + 8u16).into(),
                ident: 0u16.into(),
                flags_frag: 0u16.into(),
                ttl: 255u8.into(),
                protocol: 17u8.into(),
                header_checksum: 0u16.into(),
                src: [192, 168, 0, 2],
                dst: [224, 0, 0, 251],
            };
            let udp = UdpHeaderWire {
                src_port: 5353u16.into(),
                dst_port: 5353u16.into(),
                len: 8u16.into(),
                checksum: 0u16.into(),
            };
            let mut frame = Vec::new();
            write_specific(&mut frame, &eth).unwrap();
            write_specific(&mut frame, &ip).unwrap();
            write_specific(&mut frame, &udp).unwrap();
            frames.push(frame);
        }

        // 1) ARP request.
        {
            let eth = Ethernet2HeaderWire {
                dst: [0xff; 6],
                src: [0x02, 0x00, 0x00, 0x00, 0x00, 0x02],
                ethertype: 0x0806u16.into(),
            };
            let arp = ArpHeaderWire {
                htype: 1u16.into(),
                ptype: 0x0800u16.into(),
                hlen: 6u8.into(),
                plen: 4u8.into(),
                oper: 1u16.into(),
                sender_hw: eth.src,
                sender_ip: [192, 168, 0, 10],
                target_hw: [0u8; 6],
                target_ip: [192, 168, 0, 1],
            };
            let mut frame = Vec::new();
            write_specific(&mut frame, &eth).unwrap();
            write_specific(&mut frame, &arp).unwrap();
            frames.push(frame);
        }

        // 2) IPv4 TCP SYN (pretend HTTP).
        {
            let eth = Ethernet2HeaderWire {
                dst: [0x10, 0x20, 0x30, 0x40, 0x50, 0x60],
                src: [0x02, 0x00, 0x00, 0x00, 0x00, 0x03],
                ethertype: 0x0800u16.into(),
            };
            let ip = Ipv4HeaderWire {
                version_ihl: 0x45u8.into(),
                dscp_ecn: 0u8.into(),
                total_len: (20u16 + 20u16).into(),
                ident: 0x1234u16.into(),
                flags_frag: 0u16.into(),
                ttl: 64u8.into(),
                protocol: 6u8.into(),
                header_checksum: 0u16.into(),
                src: [10, 0, 0, 2],
                dst: [93, 184, 216, 34],
            };
            let tcp = TcpHeaderWire {
                src_port: 51515u16.into(),
                dst_port: 80u16.into(),
                seq: 1u32.into(),
                ack: 0u32.into(),
                // data offset=5 (20 bytes), flags=SYN.
                data_offset_reserved_flags: ((5u16 << 12) | 0x0002u16).into(),
                window: 65535u16.into(),
                checksum: 0u16.into(),
                urgent: 0u16.into(),
            };
            let mut frame = Vec::new();
            write_specific(&mut frame, &eth).unwrap();
            write_specific(&mut frame, &ip).unwrap();
            write_specific(&mut frame, &tcp).unwrap();
            frames.push(frame);
        }

        // 3) IPv6 TCP SYN (pretend HTTPS) with a Hop-by-Hop extension header.
        {
            let eth = Ethernet2HeaderWire {
                dst: [0x10, 0x11, 0x12, 0x13, 0x14, 0x15],
                src: [0x02, 0x00, 0x00, 0x00, 0x00, 0x04],
                ethertype: 0x86DDu16.into(),
            };
            let ip6 = Ipv6HeaderWire {
                ver_tc_flow: 0x6000_0000u32.into(),
                payload_len: (8u16 + 20u16).into(),
                next_header: 0u8.into(),
                hop_limit: 64u8.into(),
                src: [0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
                dst: [0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2],
            };
            let tcp = TcpHeaderWire {
                src_port: 51516u16.into(),
                dst_port: 443u16.into(),
                seq: 1u32.into(),
                ack: 0u32.into(),
                data_offset_reserved_flags: ((5u16 << 12) | 0x0002u16).into(),
                window: 65535u16.into(),
                checksum: 0u16.into(),
                urgent: 0u16.into(),
            };
            let mut frame = Vec::new();
            write_specific(&mut frame, &eth).unwrap();
            write_specific(&mut frame, &ip6).unwrap();
            // Hop-by-Hop header: next=TCP(6), hdr_ext_len=0 => 8 bytes total.
            frame.extend_from_slice(&[6u8, 0u8]);
            frame.extend_from_slice(&[0u8; 6]);
            write_specific(&mut frame, &tcp).unwrap();
            frames.push(frame);
        }

        // 4) VLAN-tagged IPv4 UDP (pretend DNS).
        {
            let eth = Ethernet2HeaderWire {
                dst: [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff],
                src: [0x02, 0x00, 0x00, 0x00, 0x00, 0x05],
                ethertype: 0x8100u16.into(),
            };
            let tag = VlanTagWire {
                // VLAN ID 42
                tci: 42u16.into(),
                ethertype: 0x0800u16.into(),
            };
            let ip = Ipv4HeaderWire {
                version_ihl: 0x45u8.into(),
                dscp_ecn: 0u8.into(),
                total_len: (20u16 + 8u16).into(),
                ident: 0u16.into(),
                flags_frag: 0u16.into(),
                ttl: 64u8.into(),
                protocol: 17u8.into(),
                header_checksum: 0u16.into(),
                src: [192, 168, 42, 10],
                dst: [192, 168, 42, 1],
            };
            let udp = UdpHeaderWire {
                src_port: 53000u16.into(),
                dst_port: 53u16.into(),
                len: 8u16.into(),
                checksum: 0u16.into(),
            };
            let mut frame = Vec::new();
            write_specific(&mut frame, &eth).unwrap();
            write_specific(&mut frame, &tag).unwrap();
            write_specific(&mut frame, &ip).unwrap();
            write_specific(&mut frame, &udp).unwrap();
            frames.push(frame);
        }

        frames
    }

    pub fn run() -> io::Result<()> {
        let mut args = std::env::args().skip(1);
        let mut input_path: Option<String> = None;
        let mut write_demo: Option<String> = None;
        let mut write_demo_pcap: Option<String> = None;
        let mut pcap_mode = false;
        let mut demo_mode = false;

        while let Some(a) = args.next() {
            match a.as_str() {
                "--demo" => {
                    demo_mode = true;
                }
                "--demo-pcap" => {
                    write_demo_pcap = args.next();
                    if write_demo_pcap.is_none() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "--demo-pcap needs a path",
                        ));
                    }
                }
                "--pcap" => {
                    pcap_mode = true;
                    input_path = args.next();
                    if input_path.is_none() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "--pcap needs a path",
                        ));
                    }
                }
                "--write-demo" => {
                    write_demo = args.next();
                    if write_demo.is_none() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "--write-demo needs a path",
                        ));
                    }
                }
                "-h" | "--help" => {
                    println!(
                        "ethernet_inspector [--demo] [--demo-pcap <out.pcap>] [--pcap <capture.pcap>] [--write-demo <out.bin>] [<in.bin>]\n\nModes:\n  --demo              Generate a few mock frames in-process and print decoded summaries\n  --demo-pcap <path>  Write the same mock frames as a classic PCAP (Ethernet linktype)\n\nInput formats:\n  * default: repeated (u16be len + len bytes)\n  * --pcap: classic pcap (DLT_EN10MB Ethernet only)\n\nIf <in.bin> is omitted (and not using --pcap), reads from stdin."
                    );
                    return Ok(());
                }
                _ => {
                    if input_path.is_none() {
                        input_path = Some(a);
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "too many positional args",
                        ));
                    }
                }
            }
        }

        if let Some(out) = write_demo {
            let frames = make_demo_frames();
            let mut f = std::fs::File::create(out)?;
            write_frames(&mut f, &frames)?;
            return Ok(());
        }

        if let Some(out) = write_demo_pcap {
            let frames = make_demo_frames();
            let mut f = std::fs::File::create(out)?;
            write_pcap(&mut f, &frames)?;
            return Ok(());
        }

        if demo_mode {
            let frames = make_demo_frames();
            for (i, f) in frames.iter().enumerate() {
                println!("{:04}: {}", i, parse_eth_frame(f));
            }
            return Ok(());
        }

        let frames = if let Some(p) = input_path {
            let f = std::fs::File::open(p)?;
            if pcap_mode {
                read_pcap(f)?
            } else {
                read_frames(f)?
            }
        } else {
            let stdin = std::io::stdin();
            read_frames(stdin.lock())?
        };

        for (i, f) in frames.iter().enumerate() {
            println!("{:04}: {}", i, parse_eth_frame(f));
        }

        Ok(())
    }
}

fn main() {
    #[cfg(all(feature = "derive", feature = "io-std"))]
    {
        if let Err(e) = demo::run() {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }

    #[cfg(not(all(feature = "derive", feature = "io-std")))]
    eprintln!(
        "This example requires features: derive, io-std\n\n  cargo run --example ethernet_inspector --features \"derive io-std\" -- <file>"
    );
}
