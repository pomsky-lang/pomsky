use std::{path::PathBuf, process::Stdio, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{ChildStdin, ChildStdout, Command},
    sync::Mutex,
};

use crate::files::TestResult;

type Out = Lines<BufReader<ChildStdout>>;
type Data = (ChildStdin, Out, usize);

#[derive(Clone)]
pub(crate) struct Processes {
    js: Arc<Mutex<Option<Data>>>,
    java: Arc<Mutex<Option<Data>>>,
}

impl Processes {
    pub(crate) fn new() -> Self {
        Processes { js: Arc::new(Mutex::new(None)), java: Arc::new(Mutex::new(None)) }
    }

    fn start_process(program: &str, dir: &str, args: &[&str]) -> (ChildStdin, Out) {
        let mut abs_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        abs_dir.push(dir);
        let mut process = Command::new(program)
            .current_dir(&abs_dir)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn");

        let stdin = process.stdin.take().expect("no handle to stdin");
        let stdout = process.stdout.take().expect("no handle to stdout");

        let reader = BufReader::new(stdout).lines();

        (stdin, reader)
    }

    async fn test_data(&self, data: &Mutex<Option<Data>>, regex: impl Into<String>) -> TestResult {
        let mut lock = data.lock().await;
        let (stdin, stdout, count) = (*lock).as_mut().unwrap();

        *count += 1;
        stdin.write_all((regex.into() + "\n").as_bytes()).await.unwrap();

        let line = stdout.next_line().await.unwrap().unwrap();
        if line == "success" {
            TestResult::Success
        } else {
            TestResult::InvalidOutput(line.replace(r"\n", "\n").replace(r"\\", "\\"))
        }
    }

    async fn spawn_js(&self) {
        let mut guard = self.js.lock().await;
        match &mut *guard {
            Some(_) => {}
            place @ None => {
                let (stdin, reader) =
                    Self::start_process("node", "tests/js", &["regex-tester-async.js"]);
                *place = Some((stdin, reader, 0));
            }
        }
    }

    async fn spawn_java(&self) {
        let mut guard = self.java.lock().await;
        match &mut *guard {
            Some(_) => {}
            place @ None => {
                let (stdin, reader) =
                    Self::start_process("java", "tests/java", &["RegexTesterAsync"]);
                *place = Some((stdin, reader, 0));
            }
        }
    }

    pub(crate) async fn test_js(&self, regex: impl Into<String>) -> TestResult {
        self.spawn_js().await;
        self.test_data(&self.js, regex).await
    }

    pub(crate) async fn test_java(&self, regex: impl Into<String>) -> TestResult {
        self.spawn_java().await;
        self.test_data(&self.java, regex).await
    }

    pub(crate) async fn get_js_count(&self) -> usize {
        let guard = self.js.lock().await;
        match &*guard {
            &Some((_, _, count)) => count,
            _ => 0,
        }
    }

    pub(crate) async fn get_java_count(&self) -> usize {
        let guard = self.java.lock().await;
        match &*guard {
            &Some((_, _, count)) => count,
            _ => 0,
        }
    }
}
