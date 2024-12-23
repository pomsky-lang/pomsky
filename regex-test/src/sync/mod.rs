use std::{io, path::Path, process::Command, thread};

use crate::Outcome;

use super::count::Count;
use process::Process;

mod process;

#[derive(Default)]
pub struct RegexTest {
    pub js: Process,
    pub java: Process,
    pub py: Process,
    pub dotnet: Process,
    pub rust: Count,
    pub pcre: Count,
    pub ruby: Count,
    pub re2: Count,
}

impl RegexTest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init_processes(&self) {
        thread::scope(|scope| {
            scope.spawn(|| self.test_js("x"));
            scope.spawn(|| self.test_java("x"));
            scope.spawn(|| self.test_python("x"));
            scope.spawn(|| self.test_dotnet("x"));
        });
        self.js.reset_count();
        self.java.reset_count();
        self.py.reset_count();
        self.dotnet.reset_count();
    }

    pub fn kill_processes(&self) -> io::Result<()> {
        self.js.kill()?;
        self.py.kill()?;
        self.java.kill()?;
        self.dotnet.kill()?;
        Ok(())
    }

    pub fn test_rust(&self, regex: &str) -> Outcome {
        self.rust.add_one();
        crate::native::rust(regex, &[] as &[&str])
    }

    pub fn test_rust_with(&self, regex: &str, tests: &[impl AsRef<str>]) -> Outcome {
        self.rust.add_one();
        crate::native::rust(regex, tests)
    }

    pub fn test_pcre(&self, regex: &str) -> Outcome {
        self.pcre.add_one();
        crate::native::pcre(regex, &[] as &[&str])
    }

    pub fn test_pcre_with(&self, regex: &str, tests: &[impl AsRef<str>]) -> Outcome {
        self.pcre.add_one();
        crate::native::pcre(regex, tests)
    }

    pub fn test_ruby(&self, regex: &str) -> Outcome {
        self.ruby.add_one();
        crate::native::ruby(regex, &[] as &[&str])
    }

    pub fn test_ruby_with(&self, regex: &str, tests: &[impl AsRef<str>]) -> Outcome {
        self.ruby.add_one();
        crate::native::ruby(regex, tests)
    }

    pub fn test_js(&self, regex: impl AsRef<str>) -> Outcome {
        self.test_js_with(regex, &[] as &[&str])
    }

    pub fn test_js_with(&self, regex: impl AsRef<str>, tests: &[impl AsRef<str>]) -> Outcome {
        self.js.start("js", "deno", &["run", "tester-deno-async.js"]);
        self.js.test(regex, tests)
    }

    pub fn test_python(&self, regex: impl AsRef<str>) -> Outcome {
        self.test_python_with(regex, &[] as &[&str])
    }

    pub fn test_python_with(&self, regex: impl AsRef<str>, tests: &[impl AsRef<str>]) -> Outcome {
        self.py.start("python", "python", &["tester_async.py"]);
        self.py.test(regex, tests)
    }

    pub fn test_java(&self, regex: impl AsRef<str>) -> Outcome {
        self.test_java_with(regex, &[] as &[&str])
    }

    pub fn test_java_with(&self, regex: impl AsRef<str>, tests: &[impl AsRef<str>]) -> Outcome {
        self.java.start_with("java", "java", &["TesterAsync"], || {
            let compiled = concat!(env!("CARGO_MANIFEST_DIR"), "/java/TesterAsync.class");
            if !Path::new(compiled).exists() {
                let result = Command::new("javac")
                    .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/java"))
                    .arg("TesterAsync.java")
                    .output()
                    .expect("`javac` executable not found, required for running Java regex flavor tests");
                assert!(result.status.success(), "Could not compile Java file");
            }
        });

        self.java.test(regex, tests)
    }

    pub fn test_dotnet(&self, regex: impl AsRef<str>) -> Outcome {
        self.test_dotnet_with(regex, &[] as &[&str])
    }

    pub fn test_dotnet_with(&self, regex: impl AsRef<str>, tests: &[impl AsRef<str>]) -> Outcome {
        self.dotnet.start("dotnet", "dotnet-script", &["TesterAsync.cs"]);
        self.dotnet.test(regex, tests)
    }

    pub fn test_re2(&self, regex: &str) -> Outcome {
        self.re2.add_one();
        crate::native::re2(regex, &[] as &[&str])
    }

    pub fn test_re2_with(&self, regex: &str, tests: &[impl AsRef<str>]) -> Outcome {
        self.re2.add_one();
        crate::native::re2(regex, tests)
    }
}
