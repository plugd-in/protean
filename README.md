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
* Safe methods for checked conversion of integer/float types.
* `Cow` for slice-type variants (`Bytes` and `Text`) for
  zero-copy deserialization.
* (`serde` feature) De/serialization via serde.
* (`owned` feature) Self-referential `OwnedDataCell` which
  supports deserializing from an owned buffer without
  copying, using the `yoke` crate.

It's surprisingly hard to do zero-copy deserialization from an
owned buffer right, as you essentially need a `'self` lifetime to
borrow from self, which requires either `Box::leak(...)` or
`Pin<Box<...>>` to get a static lifetime. Then, once you have a static
lifetime to self-referenced data, you need to narrow the static
lifetime to preserve soundness. Thankfully `yoke` handles the
safety of self-reference and lifetime narrowing which avoids
needing to manually ensure soundness of self-referential data.

## License
Licensed under the MIT license.
