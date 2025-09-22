use std::{
    io::{self, BufRead, BufReader, Lines, Write},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio},
    sync::Mutex,
};

use crate::Outcome;

type Out = Lines<BufReader<ChildStdout>>;

type Data = (Child, ChildStdin, Out, usize);

#[derive(Default)]
pub struct Process {
    data: Mutex<Option<Data>>,
}

impl Process {
    pub(crate) fn start(
        &self,
        dir: &'static str,
        program: &'static str,
        args: &'static [&'static str],
    ) {
        self.start_with(dir, program, args, || ());
    }

    pub(crate) fn start_with(
        &self,
        dir: &'static str,
        program: &'static str,
        args: &'static [&'static str],
        before: impl FnOnce(),
    ) {
        let mut guard = self.data.lock().unwrap();
        match &mut *guard {
            Some(_) => {}
            place @ None => {
                before();

                let mut abs_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                abs_dir.push(dir);
                let mut process = Command::new(program)
                    .current_dir(&abs_dir)
                    .args(args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("failed to spawn child");

                let stdin = process.stdin.take().expect("no handle to stdin");
                let stdout = process.stdout.take().expect("no handle to stdout");

                let reader = BufReader::new(stdout).lines();

                *place = Some((process, stdin, reader, 0));
            }
        }
    }

    pub(crate) fn test(&self, regex: impl AsRef<str>, tests: &[impl AsRef<str>]) -> Outcome {
        let mut lock = self.data.lock().unwrap();
        let (_, stdin, stdout, count) = (*lock).as_mut().expect("process isn't running");

        *count += 1;
        stdin.write_all(("REGEX:".to_string() + regex.as_ref() + "\n").as_bytes()).unwrap();

        let line = stdout.next().expect("child process did not respond").unwrap();
        if line != "success" {
            return Outcome::Error(substitute_lf(&line));
        }

        for test in tests {
            stdin.write_all(("TEST:".to_string() + test.as_ref() + "\n").as_bytes()).unwrap();
            let line = stdout.next().expect("child process did not respond").unwrap();
            if line != "test good" {
                return Outcome::Error(substitute_lf(&line));
            }
        }
        stdin.write_all("\n".as_bytes()).unwrap();

        Outcome::Success
    }

    /// Returns the number of times this process was used for compiling a regex
    pub fn get_count(&self) -> usize {
        let guard = self.data.lock().unwrap();
        match &*guard {
            &Some((_, _, _, count)) => count,
            _ => 0,
        }
    }

    /// Resets the number of times this process was used for compiling a regex
    pub fn reset_count(&self) {
        let mut guard = self.data.lock().unwrap();
        if let Some((_, _, _, count)) = &mut *guard {
            *count = 0
        }
    }

    /// Sends a SIGKILL signal to the child process
    pub fn kill(&self) -> io::Result<()> {
        let mut guard = self.data.lock().unwrap();
        if let Some((child, ..)) = &mut *guard { child.kill() } else { Ok(()) }
    }

    /// Waits for the child process to exit normally
    pub fn wait(&self) -> io::Result<Option<ExitStatus>> {
        let mut guard = self.data.lock().unwrap();
        if let Some((child, ..)) = &mut *guard { child.try_wait() } else { Ok(None) }
    }
}

fn substitute_lf(line: &str) -> String {
    line.replace(r"\n", "\n").replace(r"\\", "\\")
}
