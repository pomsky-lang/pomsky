use std::{process::Stdio, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{ChildStdin, ChildStdout, Command},
    sync::Mutex,
};

use crate::files::TestResult;

type Out = Lines<BufReader<ChildStdout>>;

#[derive(Clone)]
pub(crate) struct Processes {
    js: Arc<Mutex<Option<(ChildStdin, Out, usize)>>>,
}

impl Processes {
    pub(crate) fn new() -> Self {
        Processes { js: Arc::new(Mutex::new(None)) }
    }

    async fn spawn_js(&self) {
        let mut guard = self.js.lock().await;
        match &mut *guard {
            Some(_) => {}
            place @ None => {
                let regex_tester = concat!(env!("CARGO_MANIFEST_DIR"), "/js/regex-tester-async.js");

                let mut process = Command::new("node")
                    .arg(regex_tester)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("failed to spawn");

                let stdin = process.stdin.take().expect("no handle to stdin");
                let stdout = process.stdout.take().expect("no handle to stdout");

                let reader = BufReader::new(stdout).lines();

                *place = Some((stdin, reader, 0));
            }
        }
    }

    pub(crate) async fn test_js(&self, regex: impl Into<String>) -> TestResult {
        self.spawn_js().await;

        let mut lock = self.js.lock().await;
        let (stdin, stdout, count) = (*lock).as_mut().unwrap();

        *count += 1;
        stdin.write_all((regex.into() + "\n").as_bytes()).await.unwrap();

        let line = stdout.next_line().await.unwrap().unwrap();
        if line == "success" {
            TestResult::Success
        } else {
            TestResult::InvalidOutput(line)
        }
    }

    pub(crate) async fn get_js_count(&self) -> usize {
        let guard = self.js.lock().await;
        match &*guard {
            &Some((_, _, count)) => count,
            _ => 0,
        }
    }
}
