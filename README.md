# ksite

All in one solution for my server.

## TODO

- `units::record`: record evidence picture, audio and video in real-time.

- `units::chat`: c2c crypto. ~~try implememt the RSA~~.

- `units::chat`: room.

- `untis::qqbot`: reduce threads.

- `units::status`: speed, latency, network status. ssl cert remain.

- HTTP2? **OMG it's much more complex than I think!**

## Build with MUSL

```
# dnf install zig # for fedora linux
export CC="zig cc -target x86_64-linux-musl"
cargo +nightly build --release --target=x86_64-unknown-linux-musl
```

## License

Dual license: If `qqbot` feature is enabled, AGPL-3.0; Or it's Unlicense.
