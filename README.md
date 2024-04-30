<p align="center">
    <img src="https://github.com/cs-24-pt-10-01/thor/raw/main/docs/thor.png" width="400">
</p>
<h1 align="center">
  Thor
</h1>

Thor is a framework for collecting energy measurements from executed code in processes.

## Structure

The repository contains the following essential crates:

- lib: A library for taking measurements with RAPL
- shared-lib-sync: A static library used by the processes under test, which utilizes synchronous locking
- server: The Thor server
- shared: Shared logic

These crates are intended for experiments and testing:

- mvp: A "minimum viable product", capable of measuring energy consumption of functions in a process. It converts the measurements to joules and writes them to a CSV file. This crate can be considered a continuation of the implementation of [rapl-interface](https://github.com/cs-23-pt-9-01/rapl-interface).
- test-client: A testing client for the Thor server

## Installation

Start by downloading Rust from [rust-lang.org](https://www.rust-lang.org/tools/install).

Then, clone the repository:

```bash
git clone https://github.com/cs-24-pt-10-01/thor.git
```

## Compiling

To build the project, run:

```bash
cargo build --release
```

## Usage

Thor must run under root privileges to access the RAPL interface.

To run the Thor server on Linux, execute:

```bash
sudo ./target/release/thor-server
```

For Windows, execute the following command in PowerShell as an administrator:

```bash
.\target\release\thor-server.exe
```


