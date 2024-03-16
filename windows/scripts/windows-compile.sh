#!/usr/bin/env bash
set -e

die() {
    echo Error: $@
    exit 1
}

# build information
buildtype=debug

# Parse args
for arg in "$@"; do
    case $arg in
        --release)
            buildtype=release
            ;;
        *)
            die "Unknown flag: $arg"
            ;;
    esac
done

if [[ ! -f Cargo.toml ]]; then
    die "Can only run in the same directory as project root."
fi

if ! which rustc > /dev/null; then
    die "Rust appears to not be instaled"
fi

target='x86_64-pc-windows-gnu'

echo "Rust version: $(rustc --version)"
echo "Build type: $buildtype"
echo "Target: $target"

args=()

if [[ $buildtype == release ]]; then
    args+=(--release)
fi

cargo build --target "$target" ${args[@]}

executable="target/$target/$buildtype/ode-designer-rs.exe"
mingw_dir=/usr/lib/gcc/x86_64-w64-mingw32/10-win32
rust_toolchain_dir=/usr/local/rustup/toolchains/stable-x86_64-pc-windows-gnu/bin
final_zip="target/$target/$buildtype/windows-build.zip"

{
    echo $executable && python3 windows/scripts/mingw-ldd.py "$executable" \
    --output-format filelist\
    --dll-lookup-dirs $mingw_dir $rust_toolchain_dir
} | zip -@ -j $final_zip

[ -d vendored_python ] && zip -ur $final_zip vendored_python/
