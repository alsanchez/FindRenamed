### Info

FindRenamed compares two directory trees and tries to detect files that have been renamed. This can be used to replicate the moves in a remote backup location and thus avoid potentially expensive network transfers.

### Compile

Download and install the nightly version of Rust from http://www.rust-lang.org/

Compile with 

```sh
rustc findrenamed.rs
```

### Usage

The first argument is usually the path to a older copy of your directory, while the second one is the path to the current directory where files have been renamed.

Usage example:

```sh
findrenamed ~/Backups/2014-08-01/Projects/MyAndroidApp ~/Projects/MyAndroidApp
```

Output example:

```sh
Customer.cs was renamed to model/Customer.cs
Main.java was renamed to ui/activities/MainActivity.java
```

