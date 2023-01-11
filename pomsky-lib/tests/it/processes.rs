use std::{
    future::Future,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{ChildStdin, ChildStdout, Command},
    sync::Mutex,
};

use crate::files::TestResult;

type Out = Lines<BufReader<ChildStdout>>;

#[derive(Clone, Default)]
pub(crate) struct Processes {
    pub(crate) js: Arc<Process>,
    pub(crate) java: Arc<Process>,
    pub(crate) py: Arc<Process>,
}

impl Processes {
    async fn spawn_js(&self) {
        self.js.start("tests/js", "node", &["regex-tester-async.js"]).await;
    }

    async fn spawn_java(&self) {
        self.java
            .start_with("tests/java", "java", &["RegexTesterAsync"], async {
                let compiled =
                    concat!(env!("CARGO_MANIFEST_DIR"), "/tests/java/RegexTesterAsync.class");
                if !Path::new(compiled).exists() {
                    let result = Command::new("javac")
                        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/java"))
                        .arg("RegexTesterAsync.java")
                        .output()
                        .await
                        .unwrap();
                    assert!(result.status.success(), "Could not compile Java file");
                }
            })
            .await;
    }

    async fn spawn_py(&self) {
        self.py.start("tests/python", "python", &["regex_tester_async.py"]).await;
    }

    pub(crate) async fn test_js(&self, regex: impl Into<String>) -> TestResult {
        self.spawn_js().await;
        self.js.test(regex).await
    }

    pub(crate) async fn test_java(&self, regex: impl Into<String>) -> TestResult {
        self.spawn_java().await;
        self.java.test(regex).await
    }

    pub(crate) async fn test_py(&self, regex: impl Into<String>) -> TestResult {
        self.spawn_py().await;
        self.py.test(regex).await
    }
}

type Data = (ChildStdin, Out, usize);

#[derive(Default)]
pub(crate) struct Process {
    data: Mutex<Option<Data>>,
}

impl Process {
    async fn start(&self, dir: &'static str, program: &'static str, args: &'static [&'static str]) {
        self.start_with(dir, program, args, async {}).await
    }

    async fn start_with(
        &self,
        dir: &'static str,
        program: &'static str,
        args: &'static [&'static str],
        before: impl Future<Output = ()>,
    ) {
        let mut guard = self.data.lock().await;
        match &mut *guard {
            Some(_) => {}
            place @ None => {
                before.await;

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

                *place = Some((stdin, reader, 0));
            }
        }
    }

    async fn test(&self, regex: impl Into<String>) -> TestResult {
        let mut lock = self.data.lock().await;
        let (stdin, stdout, count) = (*lock).as_mut().expect("process isn't running");

        *count += 1;
        stdin.write_all((regex.into() + "\n").as_bytes()).await.unwrap();

        let line = stdout.next_line().await.unwrap().unwrap();
        if line == "success" {
            TestResult::Success
        } else {
            TestResult::InvalidOutput(line.replace(r"\n", "\n").replace(r"\\", "\\"))
        }
    }

    pub(crate) async fn get_count(&self) -> usize {
        let guard = self.data.lock().await;
        match &*guard {
            &Some((_, _, count)) => count,
            _ => 0,
        }
    }
}
