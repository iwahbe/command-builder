# Command-Builder
I thought that the `std::process::Command` was not ergonomic to use, mostly due
to it's transient nature. This crate facilitates an ergonomic and
reusable/printable wrapper around the `Command` process. I implement Debug
(which displays like a sh script) as well as the set of common logical operators
(`&&`, `||`, `|`, `;`) to use shell commands.

## Motivating Example
``` rust
// I was interested in bundling a set of commands, then exporting them with certain environmental variables set. 
// For debugging purposes, I wanted to see what commands had been run . I ended up with functions like this:
fn call_brew(primary_arg: String, opts: &[String], env: &HashMap);
// This would call a command like so:
// brew install (opts)* primary_arg 
// plus the env configuration. I was also handling logic like 
if test_for_brew()? {
    call_brew()
} else {
    Err(BrewNotFound)
}
// when my mental model was 
brew 2&> /null && brew command

// The last pain point was debug. I wanted non-transient commands to exist. This would allow me to collect 
// and search previous commands. 
```

This library wrappes the `std::process::Command` with a struct that holds the
information necessary to compute the command. This struct is then clonable,
printable (with debug)

## Using command-builder

``` rust
use command_builder::{Command, Single};
let grep_for = Single::new("grep").a("ip").a("-c");
let direct_input = "lorim ipsum, spelling in latin is hard.";
let latin_file = Single::new("cat").a("file_name");
let searched_file = latin_file.pipe(grep_for);
let direct_search = grep_for.input(direct_input);
// Is the file what we expect?
searched_file.run()?.stdout() == direct_seach.run()?.stdout()
// confirm the commands were right
println!("searched_file: {:?}", searched_file);
// cat file_name | grep ip -c
println!("direct_search: {:?}", direct_search);
// grep ip -c < "lorim ipsum, spelling in latin is hard."
```
