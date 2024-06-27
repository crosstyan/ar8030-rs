# README

I'm bailing. [a1ien/rusb](https://github.com/a1ien/rusb) doesn't support [async API](https://libusb.sourceforge.io/api-1.0/group__libusb__asyncio.html).
Either should I move forward with [zig](https://ziglang.org) or explore other people's design choices.

I'm not familiar with the async API design of Rust, so it would be quite challenging to implement it myself.

Due to the lifetime constraints of Rust, it would be pain in the ass to do anything useful.
