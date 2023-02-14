use dns_lookup::lookup_host;

use default_net::gateway::get_default_gateway;

use pnet::packet::ethernet::{EtherTypes, MutableEthernetPacket, EthernetPacket};
use pnet::packet::icmp::echo_reply::EchoReplyPacket;
use pnet::packet::ipv4::{self, MutableIpv4Packet, Ipv4Packet};
use pnet::packet::icmp::{IcmpCode, IcmpType};
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::{Packet, MutablePacket};
use pnet::packet::util::checksum;
use pnet_datalink::{MacAddr, NetworkInterface};
use pnet::datalink::channel;
use pnet::datalink::Channel::Ethernet;

use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::time::SystemTime;


// TODO make this not const, should be able to specify v4 or v6 with a flag
const IP_VERSION: u8 = 4;
// The minimum header length is 20 bytes, so this field would be 4.
// If you add the IP options, that's another 32 bits, and this field would be 5.
// We probably won't use them, so we'll leave it as 20.
const BUFFER_SIZE: usize = 200;
const IPV4_HEADER_LEN: u8 = 20;
const ICMP_HEADER_LEN: u8 = 8;
const TEST_DATA_LEN: u8 = 64;
const HOP_LIMIT: u8 = 64;
const ECHO_REQUEST_TYPE: u8 = 8;
const ECHO_REQUEST_CODE: u8 = 0;

fn main() {
    println!("Starting ping");

    println!("\n1. Get the hostname to send the ping to (from cli)");
    let hostname = std::env::args().nth(1).expect("no hostname");
    println!("Hostname: {}", hostname);

    println!("\n2. Get the ip addr of the hostname");
    let destination_ips: Vec<IpAddr> = lookup_host(&hostname).unwrap();
    println!("IPs: {:?}", destination_ips);

    println!("\n3. Create the ICMP packet to send to the hostname");
    // Create the packet with a buffer size of 1024 bytes.
    // Create the Ethernet (layer 2) portion of the packet
    let interfaces: Vec<NetworkInterface> = pnet::datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.name == "en0")
        .unwrap();
    let source_mac = interface.mac.unwrap();
    println!("Source Mac: {}", source_mac);
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut eth_header = MutableEthernetPacket::new(&mut buffer[..]).unwrap();
    let destination_mac_address;
    match get_default_gateway() {
        Ok(gateway) => {
            println!("Default Gateway");
            println!("\tMAC: {}", gateway.mac_addr);
            println!("\tIP: {}", gateway.ip_addr);
            destination_mac_address = MacAddr::new(
                gateway.mac_addr.octets()[0],
                gateway.mac_addr.octets()[1],
                gateway.mac_addr.octets()[2],
                gateway.mac_addr.octets()[3],
                gateway.mac_addr.octets()[4],
                gateway.mac_addr.octets()[5]
            );
            eth_header.set_destination(destination_mac_address);
        },
        Err(e) => {
            println!("Couldn't get gateway: {}", e);
        },
    }
    // println!("CURRENT TIME: {:?}", SystemTime::now());
    eth_header.set_source(source_mac);
    eth_header.set_ethertype(EtherTypes::Ipv4);
    println!("[Layer 2: ethernet] {} -> {}", eth_header.get_source(), eth_header.get_destination());

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
    println!("{}", interface);
    let source_ip = interface
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| match ip.ip() {
            IpAddr::V4(ip) => ip,
            _ => unreachable!(),
        })
        .unwrap();
    ip_header.set_source(Ipv4Addr::new(source_ip.octets()[0], source_ip.octets()[1], source_ip.octets()[2], source_ip.octets()[3]));
    // Set destination to the ip address from DNS lookup above
    let destination_ip = destination_ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| match ip {
            IpAddr::V4(ip) => ip,
            _ => unreachable!(),
        })
        .unwrap();
    ip_header.set_destination(Ipv4Addr::new(destination_ip.octets()[0], destination_ip.octets()[1], destination_ip.octets()[2], destination_ip.octets()[3]));
    ip_header.set_checksum(ipv4::checksum(&ip_header.to_immutable()));
    println!("[Layer 3: IP] {} -> {}", ip_header.get_source(), ip_header.get_destination());

    let mut echo_request_header = MutableEchoRequestPacket::new(ip_header.payload_mut()).unwrap();
    echo_request_header.set_icmp_type(IcmpType::new(ECHO_REQUEST_TYPE));
    echo_request_header.set_icmp_code(IcmpCode::new(ECHO_REQUEST_CODE));
    echo_request_header.set_identifier(1234 as u16);
    echo_request_header.set_sequence_number(0 as u16);
    echo_request_header.set_checksum(checksum(echo_request_header.packet(), 1));
    println!("[Layer 4: ICMP] Echo Request: [Code: {}, Type: {}, Id: {}, Seq: {}]",
        echo_request_header.get_icmp_type().0,
        echo_request_header.get_icmp_code().0,
        echo_request_header.get_identifier(),
        echo_request_header.get_sequence_number());

    println!("\n4. Send the packet and start a timer");
    // Create a channel to send on
    let (mut tx, mut rx) = match channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Unable to create channel: {}", e),
    };
    let now = SystemTime::now();
    tx.send_to(&buffer, None);
    // This is to process the echo request packet
    let _ = match rx.next() {
        Ok(packet) => packet,
        Err(e) => panic!("{}", e),
    };

    println!("\n5. Get the response and end the timer and print the timer difference");
    let echo_reply_raw_packet = match rx.next() {
        Ok(packet) => packet,
        Err(e) => panic!("{}", e),
    };
    match now.elapsed() {
        Ok(t) => println!("Latency: {}ms", t.as_millis()),
        Err(e) => panic!("Unable to get duration: {}", e),
    };
    let l2_echo_reply_packet = EthernetPacket::new(echo_reply_raw_packet)
        .unwrap();
    let l3_echo_reply_packet = Ipv4Packet::new(l2_echo_reply_packet.payload())
        .unwrap();
    let echo_reply_packet = EchoReplyPacket::new(l3_echo_reply_packet.payload())
        .unwrap();
    println!("Echo Reply: [Code: {}, Type: {}, Id: {}, Seq: {}]",
        echo_reply_packet.get_icmp_code().0,
        echo_reply_packet.get_icmp_type().0,
        echo_reply_packet.get_identifier(),
        echo_reply_packet.get_sequence_number());
}
