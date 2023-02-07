use dns_lookup::lookup_host;

use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket};
use pnet::packet::ipv4::{self, Ipv4Packet, MutableIpv4Packet};
use pnet::packet::icmp::{IcmpCode, IcmpType, IcmpPacket, MutableIcmpPacket};
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::{Packet, MutablePacket};
use pnet::packet::util::checksum;
use pnet_datalink::{MacAddr, NetworkInterface};
use pnet::datalink::channel;
use pnet::datalink::Channel::Ethernet;

use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::time::{Duration, SystemTime};


// TODO make this not static, should be able to specify v4 or v6 with a flag
static IP_VERSION: u8 = 4;
// The minimum header length is 20 bytes, so this field would be 4.
// If you add the IP options, that's another 32 bits, and this field would be 5.
// We probably won't use them, so we'll leave it as 20.
static IPV4_HEADER_LEN: u8 = 20;
static ICMP_HEADER_LEN: u8 = 8;
static TEST_DATA_LEN: u8 = 64;
static HOP_LIMIT: u8 = 64;
static ECHO_REQUEST_TYPE: u8 = 8;
static ECHO_REQUEST_CODE: u8 = 0;

fn main() {
    println!("Starting ping");

    println!("\n1. Get the hostname to send the ping to (from cli)");
    let hostname = std::env::args().nth(1).expect("no hostname");
    println!("Hostname: {}", hostname);

    println!("\n2. Get the ip addr of the hostname");
    let ips: Vec<IpAddr> = lookup_host(&hostname).unwrap();
    println!("IPs: {:?}", ips);

    println!("\n3. Create the ICMP packet to send to the hostname");
    // Create the packet with a buffer size of 1024 bytes.
    // Create the Ethernet (layer 2) portion of the packet
    let interfaces: Vec<NetworkInterface> = pnet::datalink::interfaces();
    // This printline is basically the same as ifconfig
    // println!("Interfaces: {:?}", interfaces);
    let interface = interfaces
        .into_iter()
        // TODO Don't hardcode this
        .find(|iface| iface.name == "en0")
        .unwrap();
    let source_ip = interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| match ip.ip() {
            IpAddr::V4(ip) => ip,
            _ => unreachable!(),
        })
        .unwrap();
    let source_mac = interface.mac.unwrap();
    println!("Source Mac: {}", source_mac);
    // TODO Make buffer size a const
    let mut buffer = [0u8; 200];
    let mut eth_header = MutableEthernetPacket::new(&mut buffer[..]).unwrap();
    // TODO Don't hardcode this
    eth_header.set_destination(MacAddr(0xe4, 0xf0, 0x42, 0xce, 0xf7, 0x48));
    eth_header.set_source(source_mac);
    eth_header.set_ethertype(EtherTypes::Ipv4);
    println!("[Layer 2: en0] {} -> {}", eth_header.get_source(), eth_header.get_destination());

    // Build the IP portion (layer 3) of the packet on top of the ICMP packet
    let mut ip_header = MutableIpv4Packet::new(eth_header.payload_mut()).unwrap();
    ip_header.set_version(IP_VERSION);
    // This field configures how much of the packet is IP header in 32-bit increments.
    // So it's basically num bytes of header / 4.
    ip_header.set_header_length(IPV4_HEADER_LEN / 4);
    ip_header.set_total_length((IPV4_HEADER_LEN + ICMP_HEADER_LEN + TEST_DATA_LEN) as u16);
    // This field is decremented every time it is passed on from a router.
    // So effectively, this is the number of hops that a packet can take before it is dropped.
    ip_header.set_ttl(HOP_LIMIT);
    ip_header.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
    ip_header.set_source(Ipv4Addr::new(192, 168, 86, 20));
    // Set destination to the ip address from DNS lookup above
    // TODO Don't hardcode this
    ip_header.set_destination(Ipv4Addr::new(142, 251, 46, 174));
    ip_header.set_checksum(ipv4::checksum(&ip_header.to_immutable()));
    println!("[Layer 3: IP] {} -> {}", ip_header.get_source(), ip_header.get_destination());

    // let mut icmp_header = MutableIcmpPacket::new(ip_header.payload_mut()).unwrap();
    // icmp_header.set_icmp_type(IcmpType::new(ECHO_REQUEST_TYPE));
    // icmp_header.set_icmp_code(IcmpCode::new(ECHO_REQUEST_CODE));
    // icmp_header.set_identifier(1234 as u16);
    // icmp_header.set_sequence_number(0 as u16);
    // // TODO Set data if needed
    // icmp_header.set_checksum(checksum(&icmp_header.to_immutable()));
    let mut echo_request_header = MutableEchoRequestPacket::new(ip_header.payload_mut()).unwrap();
    echo_request_header.set_icmp_type(IcmpType::new(ECHO_REQUEST_TYPE));
    echo_request_header.set_icmp_code(IcmpCode::new(ECHO_REQUEST_CODE));
    echo_request_header.set_identifier(1234 as u16);
    echo_request_header.set_sequence_number(0 as u16);
    // TODO
    echo_request_header.set_checksum(checksum(echo_request_header.packet(), 1));

    println!("[Layer 4: ICMP] Echo Request");

    println!("\n4. Send the packet and start a timer");
    // Create a channel to send on
    let mut tx = match channel(&interface, Default::default()) {
        Ok(Ethernet(tx, _)) => tx,
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Unable to create channel: {}", e),
    };
    tx.send_to(&buffer, None);

    println!("\n5. Get the response and end the timer and print the timer difference");
}
