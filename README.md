# rust-metamath
A Metamath verifier written in rust


other verifier's that I will probably take inspiration from.


https://github.com/david-a-wheeler/metamath-knife

https://github.com/david-a-wheeler/mmverify.py






# Running

Compile it with `cargo build --release`, the binary should be in `/target/release`

Change to the correct directory then do `path/to/binary name-of-mm-file.mm`




# Features/Goals

The main goal of this project is to provide a faster metamath checker than the
default, as we've had some issues relating to its performance.

## Potential Features

Eventually we would like to provide some extra features to metamath itself.  It
may be matching logic specific. 

Incorporating parallelism to the verifying part would be nice, and I will
probably take inspiration from the metamath knife repo. I noticed that they also
parallelize the parsing, which I'm probably not going to implement because I
think the verifying part is the slow part, and it won't matter if the parsing is
already fast enough. 

## Why Rust

Rust is a very fast language, the only viable alternatives would be

