# Arrow2: Transmute-free Arrow

[![test](https://github.com/jorgecarleitao/arrow2/actions/workflows/test.yml/badge.svg)](https://github.com/jorgecarleitao/arrow2/actions/workflows/Build.yml)
[![codecov](https://codecov.io/gh/jorgecarleitao/arrow2/branch/main/graph/badge.svg?token=AgyTF60R3D)](https://codecov.io/gh/jorgecarleitao/arrow2)
[![](https://img.shields.io/crates/d/arrow2.svg)](https://crates.io/crates/arrow2)
[![](https://img.shields.io/crates/dv/arrow2.svg)](https://crates.io/crates/arrow2)
[![](https://docs.rs/arrow2/badge.svg)](https://docs.rs/arrow2/)

A Rust crate to work with [Apache Arrow](https://arrow.apache.org/).
The most feature-complete implementation of the Arrow format after the C++
implementation.

Check out [the guide](https://jorgecarleitao.github.io/arrow2/) for a general introduction
on how to use this crate, and
[API docs](https://jorgecarleitao.github.io/arrow2/docs/arrow2/index.html) for a detailed
documentation of each of its APIs.

## Features

* Most feature-complete implementation of Apache Arrow after the reference implementation (C++)
  * Float 16 unsupported (not a Rust native type)
  * Decimal 256 unsupported (not a Rust native type)
* C data interface supported for all Arrow types (read and write)
* C stream interface supported for all Arrow types (read and write)
* Full interoperability with Rust's `Vec`
* MutableArray API to work with bitmaps and arrays in-place
* Full support for timestamps with timezones, including arithmetics that take
  timezones into account
* Support to read from, and write to:
  * CSV
  * Apache Arrow IPC (all types)
  * Apache Arrow Flight (all types)
  * Apache Parquet (except deep nested types)
  * Apache Avro (all types)
  * NJSON
  * ODBC (some types)
* Extensive suite of compute operations
  * aggregations
  * arithmetics
  * cast
  * comparison
  * sort and merge-sort
  * boolean (AND, OR, etc) and boolean kleene
  * filter, take
  * hash
  * if-then-else
  * nullif
  * temporal (day, month, week day, hour, etc.)
  * window
  * ... and more ...
* Extensive set of cargo feature flags to reduce compilation time and binary size
* Fully-decoupled IO between CPU-bounded and IO-bounded tasks, allowing
  this crate to both be used in `async` contexts without blocking and leverage parallelism
* Fastest known implementation of Avro and Parquet (e.g. faster than the official 
  C++ implementations)

## Safety and Security

This crate uses `unsafe` when strictly necessary:
* when the compiler can't prove certain invariants and
* FFI

We have extensive tests over these, all of which run and pass under MIRI.
Most uses of `unsafe` fall into 3 categories:

* The Arrow format has invariants over utf8 that can't be written in safe Rust
* `TrustedLen` and trait specialization are still nightly features
* FFI

We actively monitor for vulnerabilities in Rust's advisory and either patch or mitigate
them (see e.g. `.cargo/audit.yaml` and `.github/workflows/security.yaml`).

Reading parquet and IPC currently `panic!` when they receive invalid. We are 
actively addressing this.

## Integration tests

Our tests include roundtrip against:
* Apache Arrow IPC (both little and big endian) generated by C++, Java, Go, C# and JS
  implementations.
* Apache Parquet format (in its different configurations) generated by Arrow's C++ and
  Spark's implementation
* Apache Avro generated by the official Rust Avro implementation

Check [DEVELOPMENT.md](DEVELOPMENT.md) for our development practices.

## Versioning

We use the SemVer 2.0 used by Cargo and the remaining of the Rust ecosystem,
we also use the `0.x.y` versioning, since we are iterating over the API.

## Design

This repo and crate's primary goal is to offer a safe Rust implementation of the Arrow specification.
As such, it

* MUST NOT implement any logical type other than the ones defined on the arrow specification, [schema.fbs](https://github.com/apache/arrow/blob/master/format/Schema.fbs).
* MUST lay out memory according to the [arrow specification](https://arrow.apache.org/docs/format/Columnar.html)
* MUST support reading from and writing to the [C data interface](https://arrow.apache.org/docs/format/CDataInterface.html) at zero-copy.
* MUST support reading from, and writing to, the [IPC specification](https://arrow.apache.org/docs/python/ipc.html), which it MUST verify against golden files available [here](https://github.com/apache/arrow-testing).

Design documents about each of the parts of this repo are available on their respective READMEs.

## FAQ

### Any plans to merge with the Apache Arrow project?

Maybe. The primary reason to have this repo and crate is to be able to prototype
and mature using a fundamentally different design based on a transmute-free
implementation. This requires breaking backward compatibility and loss of
features that is impossible to achieve on the Arrow repo.

Furthermore, the arrow project currently has a release mechanism that is
unsuitable for this type of work:

* A release of the Apache consists of a release of all implementations of the
  arrow format at once, with the same version. It is currently at `5.0.0`.

This implies that the crate version is independent of the changelog or its API stability,
which violates SemVer. This procedure makes the crate incompatible with
Rusts' (and many others') ecosystem that heavily relies on SemVer to constraint
software versions.

Secondly, this implies the arrow crate is versioned as `>0.x`. This places
expectations about API stability that are incompatible with this effort.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
