FROM debian:bullseye
  
RUN groupadd --gid 1000 jenkins && useradd -rm -d /home/jenkins -s /bin/bash -g root -G sudo -u 1000 -g 1000 jenkins

RUN apt-get update
RUN DEBIAN_FRONTEND="noninteractive" apt-get install -y devscripts quilt chrpath git wget apt-utils tzdata debhelper libudev-dev

# For rust projects
# From https://github.com/rust-lang/docker-rust/blob/77e77508828ca2da1a9b7582d079b2d77f8b9a1a/1.52.1/buster/Dockerfile
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=stable

RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='fb3a7425e3f10d51f0480ac3cdb3e725977955b2ba21c9bdac35309563b115e8' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='f263e170da938888601a8e0bc822f8b40664ab067b390cf6c4fdb1d7c2d844e7' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='de1dddd8213644cba48803118c7c16387838b4d53b901059e01729387679dd2a' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='66c03055119cecdfc20828c95429212ae5051372513f148342758bb5d0130997' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.24.1/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch} --target x86_64-unknown-linux-gnu; \
    rustup target add aarch64-unknown-linux-gnu --toolchain stable; \
    rustup install nightly; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;
# For root user
RUN mkdir /.cargo && echo "[target.aarch64-unknown-linux-gnu]" >> /.cargo/config && echo linker = \"aarch64-linux-gnu-gcc\" >> /.cargo/config
RUN DEBIAN_FRONTEND="noninteractive" apt-get install -y gcc-aarch64-linux-gnu libssl-dev pkg-config
WORKDIR /home/jenkins
USER jenkins
# For jenkins user
RUN mkdir .cargo && echo "[target.aarch64-unknown-linux-gnu]" >> .cargo/config && echo linker = \"aarch64-linux-gnu-gcc\" >> .cargo/config
