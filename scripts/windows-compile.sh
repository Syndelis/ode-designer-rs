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
mingw_dir='/usr/x86_64-w64-mingw32/bin'

{
    echo $executable & python scripts/mingw-ldd.py "$executable" \
    --output-format filelist\
    --dll-lookup-dirs $mingw_dir
} | zip -@ -j "windows-build-$buildtype.zip"
