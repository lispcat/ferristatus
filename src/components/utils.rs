use std::{error::Error, fs, path::PathBuf};

pub fn read_string_from_fs(path: &PathBuf) -> Result<String, Box<dyn Error>> {
    let contents = fs::read_to_string(path).expect("failed to read from file");
    Ok(contents.trim().to_string())
}

// pub fn pathbuf_append<'a>(
//     path: &'a mut PathBuf,
//     rest: &str,
// ) -> Result<&'a mut PathBuf, Box<dyn Error>> {
//     path.push(rest)?;
//     Ok(path)
// }

// pub trait PathBufExt {
//     fn append<'a>(&'a mut self, suffix: &str) -> &'a mut PathBuf;
// }

// impl PathBufExt for PathBuf {
//     fn append<'a>(&'a mut self, suffix: &str) -> &'a mut PathBuf {
//         self.push(suffix);
//         self
//     }
// }

// pub trait PathBufExt {
//     fn append(self, suffix: &str) -> PathBuf;
// }

// impl PathBufExt for PathBuf {
//     fn append(mut self, suffix: &str) -> PathBuf {
//         self.push(suffix);
//         self
//     }
// }
