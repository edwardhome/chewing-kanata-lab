# Ubuntu 24.04 ABI baseline; build with --platform linux/amd64.
# Development environment only: this is not a distributable IME image.
FROM ubuntu:24.04
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    build-essential ca-certificates cmake extra-cmake-modules gettext git \
    pkg-config python3 ninja-build fcitx5 fcitx5-modules-dev \
    libfcitx5core-dev libfcitx5config-dev libfcitx5utils-dev libchewing3-dev \
    file binutils && rm -rf /var/lib/apt/lists/*
WORKDIR /work
COPY . /work
CMD ["bash", "packaging/linux/ubuntu24/check_baseline.sh"]
