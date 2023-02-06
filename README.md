# Ping (in progress)
## Overview
This is a small program that implements ping from scratch using the [libpnet](https://github.com/libpnet/libpnet) library. I'm using this project as a way to learn networking basics.

The way ping works is that you give it a domain name, and the program will do the following (typically in a loop):
* Do a DNS lookup of the domain name you provided
* Construct an ICMP Echo Request packet
* Send the packet and start a timer
* The host you sent your packet to will send back an ICMP Echo Response packet. When you receive this packet, end your timer and report the latency.

### Constructing the ICMP Echo Request packet
The packet is a byte buffer with the bits laid out as expected by the different networking layers. For this program, this is what it looks like:

	|--------------------------------------------------|
	| Layer 2 Header | Layer 3 Header | Layer 4 Header |
	|--------------------------------------------------|
	| Data...										   |
	|--------------------------------------------------|

Explanation of the layers (definitely don't quote me on this):
* Layer 1 is the physical layer, which basically specifies which network interface you're using to send packets. `en0` is usually the Wifi network interface on MacBooks. You can learn about the different network interfaces by running `ifconfig -v`. This command shows you all the different network interfaces on your device and some information about them.
* Layer 2 is the datalink layer. These headers specify how you're gonna get the packet from your device to the internet. All you need to specify here are the source and destination MAC addresses. **Open question: How are you supposed to figure out the destination MAC address?**
* Layer 3 is the network layer. These headers are configuring the packet for Internet Protocol. Currently, this program is stuck to using IPv4, but I'll get to adding IPv6 functionality soon. There are several items in the headers, the code has comments explinaing most of them.
* Layer 4 is the transport layer. For this program, we're going to configure the headers for an ICMP Echo Request packet.
* And then you have your data. **I'll be straight up, I have no idea what the data is being used for here.**

## Usage
	sudo cargo r google.com

## State of the Union
* Aside from a bunch of hardcoded values, the packet is configured and sent properly (first Wireshark output below).
* The response isn't being processed by anything, but we can see the response being sent back to us via Wireshark (second Wireshark output below). Slight caveat is that the checksum that we get _back_ is wrong. Not sure how this is even possible.
* There are a few TODOs in the code for configuring the packet the right way. Most of the items are hardcoded to google.com by running `ping google.com` on the side and using the values in those calls as seen in Wireshark (basically looking at the answer key).
* And the code is laid out horribly.

## Wireshark Output
### Echo Request
	Frame 4935: 200 bytes on wire (1600 bits), 200 bytes captured (1600 bits) on interface en0, id 0
	    Section number: 1
	    Interface id: 0 (en0)
	        Interface name: en0
	        Interface description: Wi-Fi
	    Encapsulation type: Ethernet (1)
	    Arrival Time: Feb  6, 2023 00:01:14.588897000 PST
	    [Time shift for this packet: 0.000000000 seconds]
	    Epoch Time: 1675670474.588897000 seconds
	    [Time delta from previous captured frame: 0.014224000 seconds]
	    [Time delta from previous displayed frame: 22.417739000 seconds]
	    [Time since reference or first frame: 665.953770000 seconds]
	    Frame Number: 4935
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
	    1.   .... = Flags: 0x0
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
	    Checksum: 0xf7ff [correct]
	    [Checksum Status: Good]
	    Identifier (BE): 0 (0x0000)
	    Identifier (LE): 0 (0x0000)
	    Sequence Number (BE): 0 (0x0000)
	    Sequence Number (LE): 0 (0x0000)
	    [Response frame: 4936]
	    Data (64 bytes)
	        Data: 000000000000000000000000000000000000000000000000000000000000000000000000…
	        [Length: 64]
	
### Echo Response
	Frame 4936: 106 bytes on wire (848 bits), 106 bytes captured (848 bits) on interface en0, id 0
		Section number: 1
		Interface id: 0 (en0)
			Interface name: en0
			Interface description: Wi-Fi
		Encapsulation type: Ethernet (1)
		Arrival Time: Feb  6, 2023 00:01:14.595211000 PST
		[Time shift for this packet: 0.000000000 seconds]
		Epoch Time: 1675670474.595211000 seconds
		[Time delta from previous captured frame: 0.006314000 seconds]
		[Time delta from previous displayed frame: 0.006314000 seconds]
		[Time since reference or first frame: 665.960084000 seconds]
		Frame Number: 4936
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
		Checksum: 0x0000 incorrect, should be 0xffff
			[Expert Info (Warning/Checksum): Bad checksum [should be 0xffff]]
				[Bad checksum [should be 0xffff]]
				[Severity level: Warning]
				[Group: Checksum]
		[Checksum Status: Bad]
		Identifier (BE): 0 (0x0000)
		Identifier (LE): 0 (0x0000)
		Sequence Number (BE): 0 (0x0000)
		Sequence Number (LE): 0 (0x0000)
		[Request frame: 4935]
		[Response time: 6.314 ms]
		Data (64 bytes)
			Data: 000000000000000000000000000000000000000000000000000000000000000000000000…
			[Length: 64]
