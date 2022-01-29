use crate::FailedTo;
use anyhow::{anyhow, Result};
use std::{
    fs::{copy, rename},
    path::Path,
};
use tempfile::NamedTempFile;

pub struct Backup<'path> {
    path: &'path Path,
    tempfile: NamedTempFile,
    disabled: bool,
}

impl<'path> Backup<'path> {
    pub fn new(path: &'path Path) -> Result<Self> {
        let tempfile = sibling_tempfile(path)?;
        copy(&path, tempfile.path())
            .failed_to(|| format!("copy {:?} to {:?}", path, tempfile.path()))?;
        Ok(Self {
            path,
            tempfile,
            disabled: false,
        })
    }

    pub fn disable(&mut self) {
        self.disabled = true;
    }
}

impl<'path> Drop for Backup<'path> {
    fn drop(&mut self) {
        if !self.disabled {
            rename(self.tempfile.path(), &self.path).unwrap_or_default();
        }
    }
}

fn sibling_tempfile(path: &Path) -> Result<NamedTempFile> {
    let parent = path.parent().ok_or_else(|| anyhow!("`parent` failed"))?;
    let tempfile = NamedTempFile::new_in(parent).failed_to(|| "create named temp file")?;
    Ok(tempfile)
}
