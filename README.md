# Thor

Connects to [Odin](https://github.com/zkirill/odin) via WebSockets and launches a GUI for sending and receiving messages from other connected clients. This is mostly an experiment for me to learn Rust.

```sh
$ cargo run
```

Tested on OS X and FreeBSD running Rust 1.7.0. Theoretically runs on any architecture where Rust, Cargo, and dependencies in Cargo.toml can be compiled and ran.

Messages are UTF-8 and get rendered by Freetype using font Noto Sans. If you want to see characters in languages such as Chinese then you will need to use a different font. See more options for [Noto](https://www.google.com/get/noto/).

**OS X**
![OS X](https://github.com/zkirill/thor/blob/master/screenshots/osx.png)

**FreeBSD**
![FreeBSD](https://github.com/zkirill/thor/blob/master/screenshots/freebsd.png)

Touchscreen mode in X11
-----------------------

In order to run in "touchscreen" mode in X11 without a mouse cursor first build a release binary with Cargo:

```sh
$ cargo build --release
```

Install X11 if you haven't already. Then edit `~/.xinitrc` and put 

`exec /path/to/release/binary`

Launch with:

`$ startx`.

You can exit at anytime by pressing Esc.

Thank you
---------

Code used from the following samples:

* https://github.com/cyderize/rust-websocket (MIT) Copyright (c) 2014-2015 Cyderize
* https://github.com/PistonDevelopers/conrod (MIT) Copyright (c) 2014 PistonDevelopers

**Example Assets**

- [Google Noto](https://www.google.com/get/noto/) (Apache2)

License
-------

The MIT License (MIT)

Copyright (c) 2016 Kirill Zdornyy

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
