ARG VARIANT="kinetic"
FROM ubuntu

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
     && apt-get -y install --no-install-recommends \
     curl lld lldb clangd clang build-essential git \
     ca-certificates pkg-config btop udev libudev-dev \
     libasound2-dev aria2

RUN update-ca-certificates
RUN useradd -m -G adm,users -s /bin/bash vscode