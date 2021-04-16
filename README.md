# vp9-parser

[![Latest version](https://img.shields.io/crates/v/vp9-parser.svg)](https://crates.io/crates/vp9-parser)
[![Documentation](https://docs.rs/vp9-parser/badge.svg)](https://docs.rs/vp9-parser)
![ZLIB](https://img.shields.io/badge/license-zlib-blue.svg)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

Provides tools to parse VP9 bitstreams and IVF containers.

## Use case

This crate does not contain a VP9 decoder. It only provides the tools to parse a VP9 bitstream,
which then could be handled by a dedicated decoder, e.g. `Vulkan Video`.

## Roadmap

Since the main use case of this crate is to support the usage of `Vulkan Video` for decoding VP9
videos, for which the final API for VP9 is not yet known (`VK_EXT_video_decode_VP9`), the API might
change in the future considerably.

## License

Licensed under MIT or Apache-2.0 or ZLIB.
