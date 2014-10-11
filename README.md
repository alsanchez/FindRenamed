[![Build Status](https://travis-ci.org/alsanchez/mvsync.png?branch=master)](https://travis-ci.org/alsanchez/mvsync)

### Info

Mvsync compares two directory trees and tries to detect files that have been renamed. This can be used to replicate the moves in a remote backup location and thus avoid potentially expensive network transfers.

### Compile

Download and install the nightly version of Rust and Cargo from http://crates.io/

Compile with 

```sh
cargo build
```

### Usage

The first argument is usually the path to a older copy of your directory, while the second one is the path to the current directory where files have been renamed.

Usage example:

```sh
mvsync ~/Backups/2014-08-01/Projects/MyAndroidApp ~/Projects/MyAndroidApp
```

Output example:

```sh
Customer.cs was renamed to model/Customer.cs
Main.java was renamed to ui/activities/MainActivity.java
```

