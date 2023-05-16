# Idunsid

`idunsid` is a fork of [sid-device](https://github.com/WilfredC64/sid-device), a cross-platform Network SID Device. It runs as a service on the idun-cartridge to allow SID tunes to be played from a Windows client over the network, using either emulated sound __or the real SID chip in your Commodore__.

_Note_: requires idun-cartridge software v1.0.2 or later.

Hopefully, a [demo video]() is worth a thousand words!

`idunsid` can be used by SID-players like:
[ACID 64 Player Pro](https://www.acid64.com), 
[ACID 64 console player](https://github.com/WilfredC64/acid64c) and
[JSidplay2](https://sourceforge.net/projects/jsidplay2/).

Embedded within `idunsid` is the emulation engine reSID v1.0, which is the very same SID emulator found in the Commodore emulator - Vice.

The original author started with the goal to turn a Raspberry Pi into a SID device. `idunsid` does that, but also connects to the real SID hardware via the idun-cartridge. You can use the player "client" applications listed above with your Commodore switched off, and sound will play through the headphone jack of the RPi using reSID's awesome emulation. But, with the Commodore switched on, and "Real SID" selected for output in the client, the actual SID chip is used. Furthermore, your Commodore is still accessible and can run any tools you like from the idun-shell.

Finally, multi-tasking with background SID tunes on an 8-bit Commodore!

_Tip_: When using reSID emulation. select "Fast" as the "Sampling method" for greatly improved emulation performance on the RPi.

## Development

To build the source code you need to have the following tools installed: 
[Rust](https://www.rust-lang.org/),
[Clang](https://llvm.org/). 

## Documentation

For documentation about the network SID interface, see the
[Network SID Device V4](https://htmlpreview.github.io/?https://github.com/WilfredC64/acid64c/blob/master/docs/network_sid_device_v4.html) specification,
converted from the
[JSidplay2](https://sourceforge.net/p/jsidplay2/code/HEAD/tree/trunk/jsidplay2/src/main/asciidoc/netsiddev.adoc) project.


## Thanks

Thanks to Dag Lem and all the team members and ex-team members of Vice who
helped with the SID chip emulation.

Thanks to Ken H&auml;ndel and Antti S. Lankila for creating the network SID interface that is used in JSidplay2 and JSidDevice.

Thanks to Wilfred Bos for sid-device and player applications.

## Copyright

SID Device v1.0 &ndash; Copyright &#xa9; 2021 - 2022 by Wilfred Bos

Network SID Interface &ndash; Copyright &#xa9; 2007 - 2022
by Wilfred Bos, Ken H&auml;ndel and Antti S. Lankila

reSID v1.0 &ndash; Copyright &#xa9; 1998 - 2022 by Dag Lem

idunsid v1.0 &ndash; Copyright &#xa9; 2023 by Brian Holdsworth

## Licensing

The source code is licensed under the GPL v3 license.
