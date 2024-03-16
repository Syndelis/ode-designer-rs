ARG PYTHON_BASE_APPIMAGE_PATH=/python-base-appimage
ARG WINDOWS_TARGET=x86_64-pc-windows-gnu
ARG WINDOWS_VENDORED_PYTHON_DIR=vendored_python

FROM appimagecrafters/appimage-builder AS python-appimage-downloader
ARG PYTHON_BASE_APPIMAGE_PATH

WORKDIR /downloaded-appimage

RUN wget https://github.com/niess/python-appimage/releases/download/python3.11/python3.11.1-cp311-cp311-manylinux_2_24_x86_64.AppImage -O python3.11.AppImage

RUN chmod +x ./python3.11.AppImage

RUN ./python3.11.AppImage --appimage-extract

# ------------------------------------------------------------------------------

FROM python-appimage-downloader as python-appimage
ARG PYTHON_BASE_APPIMAGE_PATH
ENV PYTHON_BASE_APPIMAGE_PATH=${PYTHON_BASE_APPIMAGE_PATH}

RUN --mount=type=cache,target=/root/.cache/pip \
    --mount=type=bind,source=requirements.txt,target=requirements.txt \
    ./squashfs-root/usr/bin/pip3 \
    install \
    -r requirements.txt

RUN cp -r ./squashfs-root $PYTHON_BASE_APPIMAGE_PATH

# ------------------------------------------------------------------------------

FROM ghcr.io/rust-lang/rust:nightly-slim AS setup
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y \
        g++ cmake pkg-config \
        libfontconfig-dev \
        file

WORKDIR /ode-designer

# ------------------------------------------------------------------------------

FROM setup AS planner

RUN cargo install cargo-chef

RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=src/,target=src/ \
    --mount=type=bind,source=crates/,target=crates/ \
    --mount=type=bind,source=.cargo/,target=.cargo/ \
    --mount=type=cache,target=/root/.cargo/ \
    cargo chef prepare --recipe-path recipe.json

# ------------------------------------------------------------------------------

FROM planner AS cacher

COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY crates/ crates/
COPY .cargo/ .cargo/

RUN --mount=type=cache,target=/root/.cargo/ \
    cargo chef cook --release --recipe-path recipe.json

# ------------------------------------------------------------------------------

FROM setup AS builder-dependencies
ARG PYTHON_BASE_APPIMAGE_PATH
ENV PYTHON_BASE_APPIMAGE_PATH=${PYTHON_BASE_APPIMAGE_PATH}

COPY --from=cacher $CARGO_HOME $CARGO_HOME

COPY --from=python-appimage $PYTHON_BASE_APPIMAGE_PATH $PYTHON_BASE_APPIMAGE_PATH

COPY --from=appimagecrafters/appimage-builder /opt/appimage-tool.AppDir /opt/appimage-tool.AppDir
RUN ln -s /opt/appimage-tool.AppDir/AppRun /usr/bin/appimagetool

# ------------------------------------------------------------------------------

FROM builder-dependencies AS builder

COPY . .

ENTRYPOINT [ "appimage/scripts/build-appimage.sh" ]

# ------------------------------------------------------------------------------

FROM python:3.11.1-slim-buster AS windows-python-bundler
ARG WINDOWS_VENDORED_PYTHON_DIR
ENV WINDOWS_VENDORED_PYTHON_DIR=${WINDOWS_VENDORED_PYTHON_DIR}

ENV PYTHON_VERSION=3.11.1
ENV SITE_PACKAGES=${WINDOWS_VENDORED_PYTHON_DIR}/Lib/site-packages

RUN apt-get update && apt-get install -y zip unzip wget

RUN wget https://bootstrap.pypa.io/get-pip.py -O get-pip.py
RUN wget https://www.python.org/ftp/python/$PYTHON_VERSION/python-$PYTHON_VERSION-embed-amd64.zip -O python.zip

RUN unzip python.zip -d $WINDOWS_VENDORED_PYTHON_DIR && \
    mkdir -p $SITE_PACKAGES

RUN --mount=type=cache,target=/root/.cache/pip \
    --mount=type=bind,source=requirements.txt,target=requirements.txt \
    pip3 install \
    --target=$SITE_PACKAGES \
    --platform=win_amd64 \
    --only-binary=:all: \
    -r requirements.txt

# So that the portable Python interpreter can discover the dependencies
RUN echo "Lib\nLib\\site-packages" >> $WINDOWS_VENDORED_PYTHON_DIR/python311._pth

# ------------------------------------------------------------------------------

FROM setup AS windows-builder
ARG WINDOWS_TARGET
ENV WINDOWS_TARGET=${WINDOWS_TARGET}

ARG WINDOWS_VENDORED_PYTHON_DIR
ENV WINDOWS_VENDORED_PYTHON_DIR=${WINDOWS_VENDORED_PYTHON_DIR}

RUN echo $WINDOWS_VENDORED_PYTHON_DIR > path.txt

COPY --from=windows-python-bundler /$WINDOWS_VENDORED_PYTHON_DIR ./$WINDOWS_VENDORED_PYTHON_DIR

RUN apt-get install -y \
    gcc-mingw-w64-x86-64 \
    g++-mingw-w64-x86-64 \
    gcc-mingw-w64-x86-64-win32-runtime \
    zip python3 python3-pefile

RUN rustup target add $WINDOWS_TARGET 
RUN rustup toolchain install stable-$WINDOWS_TARGET

COPY . .

CMD [ "windows/scripts/windows-compile.sh" ]
