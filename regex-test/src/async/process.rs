use std::{path::PathBuf, process::Stdio};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{ChildStdin, ChildStdout, Command},
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
    pub(super) async fn start(
        &self,
        dir: &'static str,
        program: &'static str,
        args: &'static [&'static str],
    ) {
        self.start_with(dir, program, args, || ()).await;
    }

    pub(super) async fn start_with(
        &self,
        dir: &'static str,
        program: &'static str,
        args: &'static [&'static str],
        before: impl FnOnce(),
    ) {
        let mut guard = self.data.lock().await;
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

    pub(super) async fn test(&self, regex: impl Into<String>) -> Outcome {
        let mut lock = self.data.lock().await;
        let (stdin, stdout, count) = (*lock).as_mut().expect("process isn't running");

        *count += 1;
        stdin.write_all((regex.into() + "\n").as_bytes()).await.unwrap();

        let line = stdout.next_line().await.expect("child process did not respond").unwrap();
        if line == "success" {
            Outcome::Success
        } else {
            Outcome::Error(line.replace(r"\n", "\n").replace(r"\\", "\\"))
        }
    }

    pub async fn get_count(&self) -> usize {
        let guard = self.data.lock().await;
        match &*guard {
            &Some((_, _, count)) => count,
            _ => 0,
        }
    }

    pub(super) async fn reset_count(&self) {
        let mut guard = self.data.lock().await;
        match &mut *guard {
            Some((_, _, count)) => *count = 0,
            _ => {}
        }
    }
}
