# IO Module driver
[![Build](https://github.com/noreya-nexus/drv-io/actions/workflows/build.yml/badge.svg)](https://github.com/noreya-nexus/drv-io/actions/workflows/build.yml)
[![Test](https://github.com/noreya-nexus/drv-io/actions/workflows/test.yml/badge.svg)](https://github.com/noreya-nexus/drv-io/actions/workflows/test.yml)

This is a user space driver written in [Rust](https://www.rust-lang.org/) for the [IO](https://noreya-nexus.tech/en/modules/io/).
It is uses the [SDBPK kernel driver](https://github.com/noreya-nexus/kernel-driver-sdbpk) and provides an API that can be accessed via a UDS socket (/run/nexus-drv-io/nexus-drv-io.socket).

The driver handles basic functions like module discovery and deployment as well as state and notification handling.

Other applications can use the interface to make high-performance use of the module functions.

**The API not yet finally released.**  
The only application using the UDS-API is the [rest-io driver](https://github.com/noreya-nexus/rest-io) which provides a [HTTP RESTful API](https://doc.noreya-nexus.tech/en/module-restful-api/io-module/).  
It is recommended to use the HTTP RESTful API for applications (network and local) as it is stable and finalized.  
If you still want to use the API you should check the source code of the [rest-io driver](https://github.com/noreya-nexus/rest-io).  

Most of the functionality is in the [rustlib-noreya-sdbp](https://github.com/noreya-nexus/rustlib-noreya-sdbp) lib.

## Building
To build this project for the target platform the "aarch64-unknown-linux-gnu" target must be installed via *rustup*.    
The "aarch64-linux-gnu-gcc" linker must also be configured (check the Dockerfile).
```
cargo build --target=aarch64-unknown-linux-gnu
```
The project can also be build directly on the Nexus if Rust is installed:
```
cargo build
```
### Docker
There is a Dockerfile in the project which allows you to build the project for arm64:
```
docker buildx build --platform linux/arm64 -t rust-cross-build .
docker run --platform linux/arm64 -t --rm -w "$PWD" -v "$PWD:$PWD":rw,z rust-cross-build cargo build --target=aarch64-unknown-linux-gnu --release
docker run --platform linux/arm64 -t --rm -w "$PWD" -v "$PWD:$PWD":rw,z rust-cross-build ./makedeb_github.sh
```

## Packaging
We do not build Debian packages on Github because the aarch64 architecture is not supported.  
Please check the [packaging guide](https://doc.noreya-nexus.tech/en/technical-details/packaging/guide/) for details.

## License
This driver is licensed under [GPLv3](LICENSE).
