# Ping (in progress)
## Overview
This is a small program that implements ping from scratch using the [libpnet](https://github.com/libpnet/libpnet) library. I'm using this project as a way to learn low-level networking basics.

The way ping works is that you give it a domain name, and the program will do the following (typically in a loop, but this code only sends the packet once):
* Do a DNS lookup of the domain name you provided
* Construct an ICMP Echo Request packet
* Send the packet and start a timer
* The host you sent your packet to will send back an ICMP Echo Response packet. When you receive this packet, end your timer and report the latency.

### Constructing the ICMP Echo Request packet
The packet is a byte buffer with the bits laid out as expected by the different networking layers. For this program, this is what it looks like:

	|--------------------------------------------------|
	| Layer 2 Header | Layer 3 Header | Layer 4 Header |
	|--------------------------------------------------|
	| Data...                                          |
	|--------------------------------------------------|

Explanation of the layers (definitely don't quote me on this):
* Layer 1 is the physical layer, which basically specifies which network interface you're using to send packets. `en0` is usually the Wifi network interface on MacBooks. You can learn about the different network interfaces by running `ifconfig -v`. This command shows you all the different network interfaces on your device and some information about them.
* Layer 2 is the datalink layer. These headers specify how you're gonna get the packet from your device to the internet. All you need to specify here are the source and destination MAC addresses. The source address can be found from `ifconfig -v` (this program uses an API equivalent). The destination address is going to be the next neighbor router to send the packet to. The most straightforward way to get this (on mac) is to run `netstat -rn` to get the routing tables, find the ip address of the default gateway, and then find the mac address from that. Here's an example:
	```
	❯ netstat -rn
	Routing tables

	Internet:
	Destination        Gateway            Flags           Netif Expire
	default            192.168.86.1       UGScg             en0
	...
	192.168.86.1       e4:f0:42:ce:f7:48  UHLWIir           en0   1175
	...
	```
	But this code doesn't do that. This code uses a library called `default-net` which uses the std:net libary to [send some UDP packets](https://github.com/shellrow/default-net/blob/main/src/gateway/unix.rs#L27) and then [parses the header of the packet that it just sent](https://github.com/shellrow/default-net/blob/main/src/socket/packet.rs#L78). This already isn't really ideal because the open source `ping` utility only sends a DNS packet and the ICMP packet. Worse still, my Wireshark shows that it sent like 250 of these UDP packets. I have no idea what's going on, but I think it's fine for a purely instructional program.
* Layer 3 is the network layer. These headers are configuring the packet for Internet Protocol. Currently, this program is stuck to using IPv4, but I'll get to adding IPv6 functionality soon. There are several items in the headers, the code has comments explinaing most of them.
* Layer 4 is the transport layer. This is where info for a TCP or UDP packet would exist. For this program though, we're going to configure the headers for an ICMP Echo Request packet.
* And then you have the data for your packet. This may contain more headers for higher network layers (e.g. HTTP).

## Usage
	cargo b && sudo cargo r google.com

## Wireshark Output
### Echo Request
	Frame 19: 200 bytes on wire (1600 bits), 200 bytes captured (1600 bits) on interface en0, id 0
		Section number: 1
		Interface id: 0 (en0)
			Interface name: en0
			Interface description: Wi-Fi
		Encapsulation type: Ethernet (1)
		Arrival Time: Feb  7, 2023 08:08:34.281684000 PST
		[Time shift for this packet: 0.000000000 seconds]
		Epoch Time: 1675786114.281684000 seconds
		[Time delta from previous captured frame: 0.058737000 seconds]
		[Time delta from previous displayed frame: 0.000000000 seconds]
		[Time since reference or first frame: 2.499944000 seconds]
		Frame Number: 19
		Frame Length: 200 bytes (1600 bits)
		Capture Length: 200 bytes (1600 bits)
		[Frame is marked: False]
		[Frame is ignored: False]
		[Protocols in frame: eth:ethertype:ip:icmp:data]
		[Coloring Rule Name: ICMP]
		[Coloring Rule String: icmp || icmpv6]
	Ethernet II, Src: Apple_b0:16:5e (c8:89:f3:b0:16:5e), Dst: Google_ce:f7:48 (e4:f0:42:ce:f7:48)
		Destination: Google_ce:f7:48 (e4:f0:42:ce:f7:48)
			Address: Google_ce:f7:48 (e4:f0:42:ce:f7:48)
			.... ..0. .... .... .... .... = LG bit: Globally unique address (factory default)
			.... ...0 .... .... .... .... = IG bit: Individual address (unicast)
		Source: Apple_b0:16:5e (c8:89:f3:b0:16:5e)
			Address: Apple_b0:16:5e (c8:89:f3:b0:16:5e)
			.... ..0. .... .... .... .... = LG bit: Globally unique address (factory default)
			.... ...0 .... .... .... .... = IG bit: Individual address (unicast)
		Type: IPv4 (0x0800)
		Trailer: 000000000000000000000000000000000000000000000000000000000000000000000000…
		Frame check sequence: 0x00000000 [unverified]
		[FCS Status: Unverified]
	Internet Protocol Version 4, Src: 192.168.86.20, Dst: 142.251.46.174
		0100 .... = Version: 4
		.... 0101 = Header Length: 20 bytes (5)
		Differentiated Services Field: 0x00 (DSCP: CS0, ECN: Not-ECT)
			0000 00.. = Differentiated Services Codepoint: Default (0)
			.... ..00 = Explicit Congestion Notification: Not ECN-Capable Transport (0)
		Total Length: 92
		Identification: 0x0000 (0)
		000. .... = Flags: 0x0
			0... .... = Reserved bit: Not set
			.0.. .... = Don't fragment: Not set
			..0. .... = More fragments: Not set
		...0 0000 0000 0000 = Fragment Offset: 0
		Time to Live: 64
		Protocol: ICMP (1)
		Header Checksum: 0xa63b [validation disabled]
		[Header checksum status: Unverified]
		Source Address: 192.168.86.20
		Destination Address: 142.251.46.174
	Internet Control Message Protocol
		Type: 8 (Echo (ping) request)
		Code: 0
		Checksum: 0xf32d [correct]
		[Checksum Status: Good]
		Identifier (BE): 1234 (0x04d2)
		Identifier (LE): 53764 (0xd204)
		Sequence Number (BE): 0 (0x0000)
		Sequence Number (LE): 0 (0x0000)
		[Response frame: 20]
		Data (64 bytes)
			Data: 000000000000000000000000000000000000000000000000000000000000000000000000…
			[Length: 64]
	
### Echo Response
	Frame 20: 106 bytes on wire (848 bits), 106 bytes captured (848 bits) on interface en0, id 0
		Section number: 1
		Interface id: 0 (en0)
			Interface name: en0
			Interface description: Wi-Fi
		Encapsulation type: Ethernet (1)
		Arrival Time: Feb  7, 2023 08:08:34.288041000 PST
		[Time shift for this packet: 0.000000000 seconds]
		Epoch Time: 1675786114.288041000 seconds
		[Time delta from previous captured frame: 0.006357000 seconds]
		[Time delta from previous displayed frame: 0.006357000 seconds]
		[Time since reference or first frame: 2.506301000 seconds]
		Frame Number: 20
		Frame Length: 106 bytes (848 bits)
		Capture Length: 106 bytes (848 bits)
		[Frame is marked: False]
		[Frame is ignored: False]
		[Protocols in frame: eth:ethertype:ip:icmp:data]
		[Coloring Rule Name: ICMP]
		[Coloring Rule String: icmp || icmpv6]
	Ethernet II, Src: Google_ce:f7:48 (e4:f0:42:ce:f7:48), Dst: Apple_b0:16:5e (c8:89:f3:b0:16:5e)
		Destination: Apple_b0:16:5e (c8:89:f3:b0:16:5e)
			Address: Apple_b0:16:5e (c8:89:f3:b0:16:5e)
			.... ..0. .... .... .... .... = LG bit: Globally unique address (factory default)
			.... ...0 .... .... .... .... = IG bit: Individual address (unicast)
		Source: Google_ce:f7:48 (e4:f0:42:ce:f7:48)
			Address: Google_ce:f7:48 (e4:f0:42:ce:f7:48)
			.... ..0. .... .... .... .... = LG bit: Globally unique address (factory default)
			.... ...0 .... .... .... .... = IG bit: Individual address (unicast)
		Type: IPv4 (0x0800)
	Internet Protocol Version 4, Src: 142.251.46.174, Dst: 192.168.86.20
		0100 .... = Version: 4
		.... 0101 = Header Length: 20 bytes (5)
		Differentiated Services Field: 0x00 (DSCP: CS0, ECN: Not-ECT)
			0000 00.. = Differentiated Services Codepoint: Default (0)
			.... ..00 = Explicit Congestion Notification: Not ECN-Capable Transport (0)
		Total Length: 92
		Identification: 0x0000 (0)
		000. .... = Flags: 0x0
			0... .... = Reserved bit: Not set
			.0.. .... = Don't fragment: Not set
			..0. .... = More fragments: Not set
		...0 0000 0000 0000 = Fragment Offset: 0
		Time to Live: 117
		Protocol: ICMP (1)
		Header Checksum: 0x713b [validation disabled]
		[Header checksum status: Unverified]
		Source Address: 142.251.46.174
		Destination Address: 192.168.86.20
	Internet Control Message Protocol
		Type: 0 (Echo (ping) reply)
		Code: 0
		Checksum: 0xfb2d [correct]
		[Checksum Status: Good]
		Identifier (BE): 1234 (0x04d2)
		Identifier (LE): 53764 (0xd204)
		Sequence Number (BE): 0 (0x0000)
		Sequence Number (LE): 0 (0x0000)
		[Request frame: 19]
		[Response time: 6.357 ms]
		Data (64 bytes)
			Data: 000000000000000000000000000000000000000000000000000000000000000000000000…
			[Length: 64]