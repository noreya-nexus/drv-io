# IO Module driver
[![Build](https://github.com/nexus-unity/drv-io/actions/workflows/build.yml/badge.svg)](https://github.com/nexus-unity/drv-io/actions/workflows/build.yml)
[![Test](https://github.com/nexus-unity/drv-io/actions/workflows/test.yml/badge.svg)](https://github.com/nexus-unity/drv-io/actions/workflows/test.yml)

This is a user space driver written in [Rust](https://www.rust-lang.org/) for the [IO Module](https://nexus-unity.com/en/modules/io/).
It is uses the [SDBPK kernel driver](https://github.com/nexus-unity/kernel-driver-sdbpk) and provides an API that can be accessed via a UDS socket (/run/nexus-drv-io/nexus-drv-io.socket).

The driver handles basic functions like module discovery and deployment as well as state and notification handling.

Other applications can use the interface to make high-performance use of the module functions.

**The API not yet finally released.**  
The only application using the UDS-API is the [rest-io driver](https://github.com/nexus-unity/rest-io) which provides a [HTTP RESTful API](https://doc.nexus-unity.com/en/module-restful-api/io-module/).  
It is recommended to use the HTTP RESTful API for applications (network and local) as it is stable and finalized.  
If you still want to use the API you should check the source code of the [rest-io driver](https://github.com/nexus-unity/rest-io).  

Most of the functionality is in the [nexus-unity-sdbp](https://github.com/nexus-unity/rustlib-nexus-unity-sdbp) lib.


## Building
To build this project for the target platform the "armv7-unknown-linux-gnueabihf" target must be installed via *rustup*.    
The "arm-linux-gnueabihf-gcc" linker must also be configured (check the Dockerfile).
```
cargo build --target=armv7-unknown-linux-gnueabihf
```
The project can also be build directly on the Nexus if Rust is installed:
```
cargo build
```
### Docker
There is a Dockerfile in the project which allows you to build the project for armv7:
```
docker build -t rust-cross-build .
docker run -t --rm -u 1000:1000 -w "$PWD" -v "$PWD:$PWD":rw,z rust-cross-build cargo build --target=armv7-unknown-linux-gnueabihf
```

## License
This driver is licensed under [GPLv3](LICENSE).
