<p align="center">
    <img src="https://github.com/cs-24-pt-10-01/thor/raw/main/docs/thor.png" width="400">
</p>
<h1 align="center">
  Thor
</h1>

Thor is a tool for collecting energy measurements of executed functions in running programs.

## Structure

The repository contains the following crates:

- lib: A library for taking measurements with RAPL
- shared-lib-sync: A static library used by the processes under test, which utilizes synchronous locking
- server: The Thor server
- shared: Shared logic

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

### Linux

Ensure that MSR registers are enabled by running:

```bash
sudo modprobe msr
```

Then, run the Thor server on Linux by executing the following command in the main directory:

```bash
sudo ./target/release/thor-server
```

### Windows

The Windows implementation makes use of the LibreHardwareMonitor's driver for accessing MSR registers. This program must be installed and running before starting the Thor server. It can be downloaded from [here](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor).

After installing the driver, start the Thor server by running the following command in Windows Terminal as an administrator in the main directory:

```bash
.\target\release\thor-server.exe
```
