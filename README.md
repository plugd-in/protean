# `protean`
A flexible data enum guaranteeing strict variant ordering
for backward-compatible binary serialization.

## Why?
I found myself making a similar data type quite often for
serialized communication of flexible values. At the same time,
when working with serialization formats that are not
self-describing, reordering enum variants is a footgun that
is easy to overlook. This crate has strict tests to maintain
variant ordering.

## Features
* Assertions to protect backward compatibility.
* `Cow` for slice-type variants (`Bytes` and `Text`) for
  zero-copy deserialization.
* Safe methods for checked conversion of integer/float types.

## License
Licensed under the MIT license.
