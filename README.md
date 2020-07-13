# Command-Builder
I thought that the `std::process::Command` was not ergonomic to use, mostly due
to it's transient nature. This crate facilitates an ergonomic and
reusable/printable wrapper around the `Command` process. I implement Debug
(which displays like a sh script) as well as the set of common logical operators
(`&&`, `||`, `|`, `;`) to use shell commands.
