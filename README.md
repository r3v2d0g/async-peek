# Read data asynchronously without removing it from the queue

[![img](https://img.shields.io/crates/l/async-peek.svg)](https://github.com/r3v2d0g/async-peek/blob/main/LICENSE.txt) [![img](https://img.shields.io/crates/v/async-peek.svg)](https://crates.io/crates/async-peek) [![img](https://docs.rs/async-peek/badge.svg)](https://docs.rs/async-peek)

This crate provides a trait to read data asynchronously without removing it from the queue (like when using the blocking methods [`std::net::TcpStream::peek()`](https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.peek) and [`std::net::UdpSocket::peek()`](https://doc.rust-lang.org/std/net/struct.UdpSocket.html#method.peek)).


## Support matrix

| name                  | tcp | udp |
|--------------------- |--- |--- |
| `smol` and `async-io` | [x] | [x] |
| `async-net`           | [x] | [x] |
| `async-std`           | [x] | [ ] |


## License

> This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy of the MPL was not distributed with this file, You can obtain one at <http://mozilla.org/MPL/2.0/>.
