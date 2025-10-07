FROM rust:1.82 AS builder

WORKDIR /usr/src/ioc
COPY . .

RUN cargo install --path .

#CMD ["ioc", "help"]


FROM debian:bookworm AS pkg-build

LABEL authors="Dr. Niko Kivel"

RUN apt-get update && \
    apt-get -y install \
    dpkg-dev \
    fakeroot \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/ioc /ioc-package/usr/bin/ioc

COPY --from=builder /usr/src/ioc/config/production.toml /ioc-package/etc/ioc/config.toml

COPY --from=builder /usr/src/ioc/templates /ioc-package/etc/ioc/templates

COPY --from=builder /usr/src/ioc/config/profile.d/ioc.sh /ioc-package/etc/profile.d/ioc.sh

COPY --from=builder /usr/src/ioc/DEBIAN /ioc-package/DEBIAN

RUN dpkg-deb --build /ioc-package && \
    dpkg-name /ioc-package.deb

FROM scratch AS package

COPY --from=pkg-build /*.deb /

# sudo docker run -it --rm -v ${PWD}:/opt/epics/build -v /opt/epics/modules/:/opt/epics/modules -u ${UID}:10001 docker.clsi.ca/cid/x86_64_debian12_epics707:latest
