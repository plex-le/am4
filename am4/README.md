# am4

This crate provides core functions that will be used in the bot/wasm bundle (brewing).

## Implementation details

Some implementation details

- `Aircrafts`, `Airports` and `Routes` are stored in a private `Vec`
    - data is deserialised from bytes with zero-copy using `rkyv`
    - indexes on the data allow for searching and suggesting
        - `HashMap<K, usize>` is first built, where `K` is an owned enum derived from columns.
        - we do not use [self referential structs](https://stackoverflow.com/questions/32300132/why-cant-i-store-a-value-and-a-reference-to-that-value-in-the-same-struct/32300133#32300133) for simplicity
        - fuzzy finding: jaro winkler the query string against every single `K`: $O(n)$
    - they are immutable and do not allow addition/deletion.
