#[cfg(test)]
mod tests {
    // TODO: Write tests
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use std::process::Stdio;
use std::{fmt, io, io::Write, process};

/// The output of a command after it has been run. Contains both stdout and stderr along with the exit code.
#[derive(Clone)]
pub struct Output {
    stderr: String,
    stdout: String,
    exit_code: i32,
}

impl Output {
    /// Test if the process finished successfully. The process is considered successful if the exit code is 0.
    pub fn success(&self) -> bool {
        self.code() == 0
    }
    /// Returns the exit code for the process.
    pub fn code(&self) -> i32 {
        self.exit_code
    }
    /// A view into standard out.
    pub fn stdout(&self) -> &str {
        self.stdout.as_ref()
    }
    /// A view into standard error.
    pub fn stderr(&self) -> &str {
        self.stderr.as_ref()
    }
}

use std::collections::HashMap;

pub trait Command: Sized + std::fmt::Debug + Clone {
    /// Equivalent to &&, as in "command 1" && "command 2".
    fn and<C: Command>(self, other: C) -> And<Self, C> {
        And {
            first: self,
            second: other,
        }
    }

    /// Equivalent to ||, as in "command 1" || "command 2".
    fn or<C: Command>(self, other: C) -> Or<Self, C> {
        Or {
            first: self,
            second: other,
        }
    }

    /// Equivalent to ;, as in "command 1"; "command 2".
    fn then<C: Command>(self, other: C) -> Then<Self, C> {
        Then {
            first: self,
            second: other,
        }
    }

    /// Equivalent to |, as in "pipe 1" | "into 2".
    fn pipe<C: Command>(self, other: C) -> Pipe<Self, C> {
        Pipe {
            first: self,
            second: other,
        }
    }

    /// Sets the env in the environment the command is run in.
    fn with_env(self, key: &str, value: &str) -> Env<Self> {
        Env {
            key: key.to_owned(),
            value: value.to_owned(),
            on: self,
        }
    }

    /// Clears the environment for non-explicitly set variables.
    fn clear_env(self) -> ClearEnv<Self> {
        ClearEnv { on: self }
    }

    /// Runs the command.
    fn run(&self) -> io::Result<Output> {
        self.run_internal(None, false, HashMap::new())
    }

    /// Pipes `input` into the following command.
    fn with_input(self, input: &str) -> Input<Self> {
        Input {
            input: input.to_owned(),
            on: self,
        }
    }

    /// The command used to define all others.
    fn run_internal(
        &self,
        input: Option<&str>,
        clear_env: bool,
        env: HashMap<String, String>,
    ) -> io::Result<Output>;
}

/// Contains input to be piped into a command.
#[derive(Clone)]
pub struct Input<F>
where
    F: Command,
{
    input: String,
    on: F,
}

impl<F: Command> std::fmt::Debug for Input<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?} < \"{}\"", self.on, self.input)
    }
}

impl<F: Command> Command for Input<F> {
    fn run_internal(
        &self,
        input: std::option::Option<&str>,
        clear_env: bool,
        env: std::collections::HashMap<std::string::String, std::string::String>,
    ) -> std::result::Result<Output, std::io::Error> {
        let input_string = match input.as_ref() {
            Some(prev) => prev.to_string() + &self.input,
            None => self.input.to_owned(),
        };
        self.on.run_internal(Some(&input_string), clear_env, env)
    }
}
impl<F: Command> fmt::Debug for ClearEnv<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "CLEAR_ENV \"{:?}\"", self.on)
    }
}
impl<F: Command> Command for ClearEnv<F> {
    fn run_internal(
        &self,
        input: std::option::Option<&str>,
        _: bool,
        env: std::collections::HashMap<std::string::String, std::string::String>,
    ) -> std::result::Result<Output, std::io::Error> {
        self.on.run_internal(input, true, env)
    }
}

/// Indicates that the environment should be cleared.
#[derive(Clone)]
pub struct ClearEnv<F>
where
    F: Command,
{
    on: F,
}

/// Adds a key-value to the calling environment.
#[derive(Clone)]
pub struct Env<F>
where
    F: Command,
{
    key: String,
    value: String,
    on: F,
}
impl<F: Command> fmt::Debug for Env<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}={} {:?}", self.key, self.value, self.on)
    }
}
impl<F: Command> Command for Env<F> {
    fn run_internal(
        &self,
        input: Option<&str>,
        clear_env: bool,
        mut env: std::collections::HashMap<std::string::String, std::string::String>,
    ) -> std::result::Result<Output, std::io::Error> {
        env.insert(self.key.clone(), self.value.clone());
        self.on.run_internal(input, clear_env, env)
    }
}

/// Allows piping of one command's standard out into another.
#[derive(Clone)]
pub struct Pipe<F, S>
where
    F: Command,
    S: Command,
{
    first: F,
    second: S,
}

impl<F: Command, S: Command> fmt::Debug for Pipe<F, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?} | {:?}", self.first, self.second)
    }
}
impl<F: Command, S: Command> Command for Pipe<F, S> {
    fn run_internal(
        &self,
        input: std::option::Option<&str>,
        clear_env: bool,
        env: std::collections::HashMap<std::string::String, std::string::String>,
    ) -> std::result::Result<Output, std::io::Error> {
        let first = self.first.run_internal(input, clear_env, env.clone())?;
        self.second
            .run_internal(Some(&first.stdout), clear_env, env)
    }
}

impl<F: Command, S: Command> fmt::Debug for Then<F, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?}; {:?}", self.first, self.second)
    }
}
impl<F: Command, S: Command> Command for Then<F, S> {
    fn run_internal(
        &self,
        input: std::option::Option<&str>,
        clear_env: bool,
        env: std::collections::HashMap<std::string::String, std::string::String>,
    ) -> std::result::Result<Output, std::io::Error> {
        self.first.run_internal(input, clear_env, env.clone())?;
        self.second.run_internal(None, clear_env, env)
    }
}

/// Executes one command, then another.
#[derive(Clone)]
pub struct Then<F, S>
where
    F: Command,
    S: Command,
{
    first: F,
    second: S,
}

impl<F: Command, S: Command> fmt::Debug for And<F, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?} && {:?}", self.first, self.second)
    }
}
impl<F: Command, S: Command> Command for And<F, S> {
    fn run_internal(
        &self,
        input: std::option::Option<&str>,
        clear_env: bool,
        env: std::collections::HashMap<std::string::String, std::string::String>,
    ) -> std::result::Result<Output, std::io::Error> {
        let first = self.first.run_internal(input, clear_env, env.clone())?;
        if first.success() {
            self.second.run_internal(None, clear_env, env)
        } else {
            Ok(first)
        }
    }
}

/// Executes one command and if successful returns the result of the other.
#[derive(Clone)]
pub struct And<F, S>
where
    F: Command,
    S: Command,
{
    first: F,
    second: S,
}

impl<F: Command, S: Command> fmt::Debug for Or<F, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{:?} || {:?}", self.first, self.second)
    }
}
impl<F: Command, S: Command> Command for Or<F, S> {
    fn run_internal(
        &self,
        input: std::option::Option<&str>,
        clear_env: bool,
        env: std::collections::HashMap<std::string::String, std::string::String>,
    ) -> std::result::Result<Output, std::io::Error> {
        let first = self.first.run_internal(input, clear_env, env.clone())?;
        if !first.success() {
            self.second.run_internal(None, clear_env, env)
        } else {
            Ok(first)
        }
    }
}

/// Executes one command and if unsuccessful returns the result of the other.
#[derive(Clone)]
pub struct Or<F, S>
where
    F: Command,
    S: Command,
{
    first: F,
    second: S,
}

/// Holds a single command to be run.
#[derive(Clone, PartialEq, Eq)]
pub struct Single(Vec<String>);
impl fmt::Debug for Single {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{}{}",
            self.0.first().unwrap(),
            self.0
                .iter()
                .skip(1)
                .fold(String::new(), |old: String, next: &String| old + " " + next)
        )
    }
}
impl Command for Single {
    fn run_internal(
        &self,
        input: Option<&str>,
        clear_env: bool,
        env: HashMap<String, String>,
    ) -> Result<Output, std::io::Error> {
        let f = self.0.first().unwrap();
        let mut out = if clear_env {
            process::Command::new(f)
                .args(self.0.iter().skip(1))
                .stderr(Stdio::piped())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .env_clear()
                .envs(env.iter())
                .spawn()?
        } else {
            process::Command::new(f)
                .args(self.0.iter().skip(1))
                .stderr(Stdio::piped())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .envs(env.iter())
                .spawn()?
        };
        if let Some(input) = input {
            write!(
                match out.stdin.as_mut() {
                    Some(i) => i,
                    None => return Err(io::Error::from(io::ErrorKind::BrokenPipe)),
                },
                "{}",
                input
            )?;
        }
        let output = out.wait_with_output()?;
        Ok(Output {
            stderr: String::from_utf8_lossy(&output.stderr).to_owned().into(),
            stdout: String::from_utf8_lossy(&output.stdout).into(),
            exit_code: output.status.code().unwrap_or(1),
        })
    }
}

impl Single {
    /// Creates a new Command which can be run in the shell.
    pub fn new(command: &str) -> Self {
        Self(vec![command.to_owned()])
    }

    /// Adds an argument to the command.
    pub fn a(mut self, argument: &str) -> Self {
        self.0.push(argument.to_owned());
        self
    }
}
