use std::{
    io::{BufRead, BufReader, Lines, Write},
    path::PathBuf,
    process::{ChildStdin, ChildStdout, Command, Stdio},
    sync::Mutex,
};

use crate::Outcome;

type Out = Lines<BufReader<ChildStdout>>;

type Data = (ChildStdin, Out, usize);

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

                *place = Some((stdin, reader, 0));
            }
        }
    }

    pub(crate) fn test(&self, regex: impl Into<String>) -> Outcome {
        let mut lock = self.data.lock().unwrap();
        let (stdin, stdout, count) = (*lock).as_mut().expect("process isn't running");

        *count += 1;
        stdin.write_all((regex.into() + "\n").as_bytes()).unwrap();

        let line = stdout.next().expect("child process did not respond").unwrap();
        if line == "success" {
            Outcome::Success
        } else {
            Outcome::Error(line.replace(r"\n", "\n").replace(r"\\", "\\"))
        }
    }

    pub fn get_count(&self) -> usize {
        let guard = self.data.lock().unwrap();
        match &*guard {
            &Some((_, _, count)) => count,
            _ => 0,
        }
    }

    pub fn reset_count(&self) {
        let mut guard = self.data.lock().unwrap();
        match &mut *guard {
            Some((_, _, count)) => *count = 0,
            _ => {}
        }
    }
}
