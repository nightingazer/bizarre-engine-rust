use std::{cell::RefCell, fs::File};

#[derive(Debug)]
pub enum LogTarget {
    Stdout,
    Stderr,
    File(Box<str>, Option<RefCell<File>>),
}

pub fn file_target(path: &str, file: Option<File>) -> LogTarget {
    LogTarget::File(
        path.into(),
        match file {
            Some(file) => Some(RefCell::new(file)),
            _ => None,
        },
    )
}

impl PartialEq for LogTarget {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LogTarget::Stdout, LogTarget::Stdout) => true,
            (LogTarget::Stderr, LogTarget::Stderr) => true,
            (LogTarget::File(path1, file1), LogTarget::File(path2, file2)) => {
                path1 == path2 && file1.is_some() && file2.is_some()
            }
            _ => false,
        }
    }
}

impl Eq for LogTarget {}
