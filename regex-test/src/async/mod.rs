use std::{path::Path, process::Command};

use crate::Outcome;

use super::count::Count;
use process::Process;

mod process;

#[derive(Default)]
pub struct RegexTest {
    pub js: Process,
    pub java: Process,
    pub py: Process,
    pub rust: Count,
    pub pcre: Count,
    pub ruby: Count,
}

impl RegexTest {
    pub fn test_rust(&self, regex: &str) -> Outcome {
        self.rust.add_one();
        crate::native::rust(regex)
    }

    pub fn test_pcre(&self, regex: &str) -> Outcome {
        self.pcre.add_one();
        crate::native::pcre(regex)
    }

    pub fn test_ruby(&self, regex: &str) -> Outcome {
        self.ruby.add_one();
        crate::native::ruby(regex)
    }

    pub async fn test_js(&self, regex: impl Into<String>) -> Outcome {
        self.js.start("js", "node", &["tester-async.js"]).await;
        self.js.test(regex).await
    }

    pub async fn test_python(&self, regex: impl Into<String>) -> Outcome {
        self.py.start("python", "python", &["tester_async.py"]).await;
        self.py.test(regex).await
    }

    pub async fn test_java(&self, regex: impl Into<String>) -> Outcome {
        self.java
            .start_with("java", "java", &["TesterAsync"], || {
                let compiled = concat!(env!("CARGO_MANIFEST_DIR"), "/java/TesterAsync.class");
                if !Path::new(compiled).exists() {
                    let result = Command::new("javac")
                        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/java"))
                        .arg("TesterAsync.java")
                        .output()
                        .unwrap();
                    assert!(result.status.success(), "Could not compile Java file");
                }
            })
            .await;

        self.java.test(regex).await
    }
}
