#[derive(Debug, Default)]
pub struct TestCases {
    runner: std::cell::RefCell<crate::RunnerSpec>,
    has_run: std::cell::Cell<bool>,
}

impl TestCases {
    pub fn new() -> Self {
        let s = Self::default();
        s.runner
            .borrow_mut()
            .include(parse_include(std::env::args_os()));
        s
    }

    /// Load tests from `glob`
    pub fn case(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner.borrow_mut().case(glob.as_ref(), None);
        self
    }

    /// Overwrite expected status for a test
    pub fn pass(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner
            .borrow_mut()
            .case(glob.as_ref(), Some(crate::CommandStatus::Pass));
        self
    }

    /// Overwrite expected status for a test
    pub fn fail(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner
            .borrow_mut()
            .case(glob.as_ref(), Some(crate::CommandStatus::Fail));
        self
    }

    /// Overwrite expected status for a test
    pub fn interrupted(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner
            .borrow_mut()
            .case(glob.as_ref(), Some(crate::CommandStatus::Interrupted));
        self
    }

    /// Overwrite expected status for a test
    pub fn skip(&self, glob: impl AsRef<std::path::Path>) -> &Self {
        self.runner
            .borrow_mut()
            .case(glob.as_ref(), Some(crate::CommandStatus::Skip));
        self
    }

    /// Set default bin, by path, for commands
    pub fn default_bin_path(&self, path: impl AsRef<std::path::Path>) -> &Self {
        let bin = Some(crate::Bin::Path(path.as_ref().into()));
        self.runner.borrow_mut().default_bin(bin);
        self
    }

    /// Set default bin, by name, for commands
    pub fn default_bin_name(&self, name: impl AsRef<str>) -> &Self {
        let bin = Some(crate::Bin::Name(name.as_ref().into()));
        self.runner.borrow_mut().default_bin(bin);
        self
    }

    /// Set default timeout for commands
    pub fn timeout(&self, time: std::time::Duration) -> &Self {
        self.runner.borrow_mut().timeout(Some(time));
        self
    }

    /// Set default environment variable
    pub fn env(&self, key: impl Into<String>, value: impl Into<String>) -> &Self {
        self.runner.borrow_mut().env(key, value);
        self
    }

    /// Run tests
    ///
    /// This will happen on `drop` if not done explicitly
    pub fn run(&self) {
        self.has_run.set(true);

        let mode = parse_mode(std::env::var_os("TRYCMD").as_deref());
        mode.initialize().unwrap();

        self.runner.borrow_mut().prepare().run(&mode);
    }
}

impl std::panic::RefUnwindSafe for TestCases {}

#[doc(hidden)]
impl Drop for TestCases {
    fn drop(&mut self) {
        if !self.has_run.get() && !std::thread::panicking() {
            self.run();
        }
    }
}

// Filter which test cases are run by trybuild.
//
//     $ cargo test -- ui trybuild=tuple_structs.rs
//
// The first argument after `--` must be the trybuild test name i.e. the name of
// the function that has the #[test] attribute and calls trybuild. That's to get
// Cargo to run the test at all. The next argument starting with `trybuild=`
// provides a filename filter. Only test cases whose filename contains the
// filter string will be run.
#[allow(clippy::needless_collect)] // false positive https://github.com/rust-lang/rust-clippy/issues/5991
fn parse_include(args: impl IntoIterator<Item = std::ffi::OsString>) -> Option<Vec<String>> {
    let filters = args
        .into_iter()
        .flat_map(std::ffi::OsString::into_string)
        .filter_map(|arg| {
            const PREFIX: &str = "trycmd=";
            if let Some(remainder) = arg.strip_prefix(PREFIX) {
                if remainder.is_empty() {
                    None
                } else {
                    Some(remainder.to_owned())
                }
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    if filters.is_empty() {
        None
    } else {
        Some(filters)
    }
}

fn parse_mode(var: Option<&std::ffi::OsStr>) -> crate::Mode {
    if var == Some(std::ffi::OsStr::new("overwrite")) {
        crate::Mode::Overwrite
    } else if var == Some(std::ffi::OsStr::new("dump")) {
        crate::Mode::Dump("dump".into())
    } else {
        crate::Mode::Fail
    }
}