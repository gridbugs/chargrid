export RUSTFLAGS="-Clink-arg=-fuse-ld=lld -Zshare-generics=y"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang
