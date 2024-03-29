# About
The code in this repository aims to solve the
[DEBS Grand Challenge 2021](https://2021.debs.org/call-for-grand-challenge-solutions/).

More details about the challenge and the solution
can be found in the [thesis](thesis.pdf) (written in german).

# Usage
## Rust Installation
This project is written in the Rust programming language.
If Rust-specific tooling is not yet installed on your machine,
visit the [rustup](https://rustup.rs/) website and follow the installation instructions.

## Setup and Configuration
**Due to the large data volume, the data is only available on request.
Contact me and we'll figure out something.**
Download the data ~~from [the shared folder](https://bwsyncandshare.kit.edu/s/caadGD4AKiHbCPR)~~
and save it into a folder. This folder will be considered the root folder of the data. 
The total dataset consists of about 32 GiB of data.

For some binaries, notably the q1 binary, `DEBS_DATA_ROOT` needs to be set
to this directory before executing the binary:
```sh
export DEBS_DATA_ROOT="/path/to/downloaded/data"
```

The data was split into smaller chunks to avoid having a multi-gigabyte file
(which seems to cause issues with nextcloud).
To extract the messages, join the parts with:
```sh
cat messages.x* > messages.tar.gz
```

Then extract the messages using:
```sh
tar xf messages.tar.gz
```

If required, the archive files can now be removed to free disk space.

## Building
### Release mode (preferred)
For the best performance, compile in release mode:
```sh
cargo build --release
```

No errors should be shown, and binaries generated in `target/release`.

### Debug mode (for development)
In debug mode, some extra assertions
are enabled which verify that the result of
optimizations is correct. As the query code is
strongly based on functional programming and relies
heavily compiler optimization, optimizations have been
enabled even in debug mode.
```sh
cargo build
```

## Running
### To solve query 1
After building, run
```sh
./target/release/q1
```

Note that `DEBS_DATA_ROOT` needs to be set
(see [Setup and Configuration](#setup-and-configuration)).

Currently, the program only outputs messages for every 5 minutes
of data processed and (although generated) does not output or
store the solution somewhere. Writing the results
somewhere is still on the todo list.


### Other binaries
These binaries are not necessary to solve the query,
but have been developed in the scope of this project.

#### `download`
The download binary was used to download the given dataset
from the DEBS servers. As the DEBS servers are offline,
this program is now only useful as a reference for how
a client would have looked like.

#### `analyze_locations`/`analyze_measurements`
Both programs analyze various featurees of the corresponding
data files and output them to stdout. They were used to
explore the data and find potential edge cases or possibilities
for performance. For details, see their source code.


#### `gen_smaller_dataset`
Generates a small test dataset which is used
to test the functionality and performance of
the location process. Require `DEBS_DATA_ROOT`
to be set correctly.

#### `geo2sql`/`batch2ql`
geo2sql and batch2sql convert location info and
batch data to SQL, respectively.

For details, see the [sql/](sql/) subfolder.


# Development
## Documentation
The code contains documentation annotations.
The full HTML documentation can be generated and viewed using
```sh
cargo doc --open --document-private-items
```

## Testing
See [TESTING.md](TESTING.md).

## Benchmarking
Currently, only one benchmark exsists,
used to analyze the speed of matchin a pair of coordinates to a city.
Use
```sh
cargo bench
```
to run all benchmarks.
