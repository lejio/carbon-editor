//! Project context: a directory or an in-memory file set, plus the buffers open from it.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::Buffer;

const MAX_FILES: usize = 4_000;
const MAX_DEPTH: usize = 16;
const MAX_BYTES: u64 = 512 * 1024;

const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "dist",
    ".next",
    ".turbo",
    ".cache",
    "coverage",
    "vendor",
];

const TEXT_EXT: &[&str] = &[
    "rs",
    "toml",
    "md",
    "txt",
    "json",
    "js",
    "jsx",
    "ts",
    "tsx",
    "mjs",
    "cjs",
    "css",
    "scss",
    "html",
    "htm",
    "vue",
    "svelte",
    "py",
    "go",
    "java",
    "kt",
    "c",
    "h",
    "cpp",
    "hpp",
    "cs",
    "rb",
    "php",
    "swift",
    "yml",
    "yaml",
    "xml",
    "svg",
    "sql",
    "sh",
    "bash",
    "zsh",
    "fish",
    "lua",
    "zig",
    "ron",
    "ini",
    "cfg",
    "conf",
    "env",
    "gitignore",
    "lock",
    "glsl",
    "wgsl",
    "vim",
];

const TEXT_NAMES: &[&str] = &[
    "readme",
    "license",
    "licence",
    "makefile",
    "dockerfile",
    "justfile",
    "gemfile",
    "rakefile",
    "procfile",
    "authors",
    "changelog",
    "copying",
];

const DOTFILES: &[&str] = &[
    ".gitignore",
    ".gitattributes",
    ".editorconfig",
    ".npmrc",
    ".nvmrc",
    ".prettierrc",
    ".eslintrc",
    ".env",
];

/// Why opening or saving a project file failed.
#[derive(Debug)]
pub enum ProjectError {
    Io(io::Error),
    /// The path is not one of this project's files.
    Missing(String),
    /// The path leaves the project root, or is not a relative file path.
    UnsafePath(String),
    /// The file is not valid UTF-8 text.
    Binary(String),
    NothingOpen,
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Missing(path) => write!(f, "no file named {path} in this project"),
            Self::UnsafePath(path) => write!(f, "{path} is outside the project"),
            Self::Binary(path) => write!(f, "{path} is not a text file"),
            Self::NothingOpen => write!(f, "no file is open"),
        }
    }
}

impl std::error::Error for ProjectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ProjectError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// One file open in the editor.
#[derive(Clone, Debug)]
pub struct Document {
    relative: String,
    buffer: Buffer,
}

impl Document {
    /// Path relative to the project root, with `/` separators.
    pub fn path(&self) -> &str {
        &self.relative
    }

    /// The last path segment.
    pub fn name(&self) -> &str {
        self.relative.rsplit('/').next().unwrap_or(&self.relative)
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
}

/// Files the editor can open, and the buffers currently open.
///
/// A directory project indexes text files under a root and reads them from
/// disk. A memory project, from [`Project::memory`], keeps the bytes itself
/// so a gallery or a test can run without a filesystem.
#[derive(Clone, Debug)]
pub struct Project {
    label: String,
    root: Option<PathBuf>,
    /// Project-relative paths, sorted.
    files: Vec<String>,
    /// Contents for a memory project. Empty when `root` is set.
    contents: BTreeMap<String, String>,
    open: Vec<Document>,
    active: Option<usize>,
    truncated: bool,
}

impl Project {
    /// An in-memory project. Paths are normalized to `/` separators.
    ///
    /// Entries that are empty or contain `..` are skipped.
    pub fn memory<I, P, C>(label: impl Into<String>, files: I) -> Self
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<String>,
        C: Into<String>,
    {
        let mut contents = BTreeMap::new();
        for (path, text) in files {
            let Some(path) = normalize_relative(&path.into()) else {
                continue;
            };
            contents.insert(path, text.into());
        }
        let files = contents.keys().cloned().collect();
        Self {
            label: label.into(),
            root: None,
            files,
            contents,
            open: Vec::new(),
            active: None,
            truncated: false,
        }
    }

    /// Index text files under `root`.
    ///
    /// Skips dependency and build directories, hidden directories, files over
    /// 512 KiB, and files whose extension is not text. The walk stops after
    /// 4,000 files; [`Self::truncated`] reports that.
    pub fn from_dir(root: impl Into<PathBuf>) -> Result<Self, ProjectError> {
        let root = root.into();
        if !root.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotADirectory,
                "project root is not a directory",
            )
            .into());
        }
        let mut files = Vec::new();
        let mut truncated = false;
        walk(&root, &root, &mut files, 0, &mut truncated)?;
        files.sort();
        files.dedup();
        let label = root
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| root.display().to_string());
        Ok(Self {
            label,
            root: Some(root),
            files,
            contents: BTreeMap::new(),
            open: Vec::new(),
            active: None,
            truncated,
        })
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// Indexed project-relative paths.
    pub fn paths(&self) -> &[String] {
        &self.files
    }

    pub fn truncated(&self) -> bool {
        self.truncated
    }

    pub fn documents(&self) -> &[Document] {
        &self.open
    }

    pub fn active_index(&self) -> Option<usize> {
        self.active
    }

    pub fn active(&self) -> Option<&Document> {
        self.active.and_then(|index| self.open.get(index))
    }

    pub fn active_mut(&mut self) -> Option<&mut Document> {
        let index = self.active?;
        self.open.get_mut(index)
    }

    pub fn active_path(&self) -> Option<&str> {
        self.active().map(Document::path)
    }

    /// Whether an open buffer for `relative` differs from its saved snapshot.
    pub fn is_dirty(&self, relative: &str) -> bool {
        self.open
            .iter()
            .any(|doc| doc.relative == relative && doc.buffer.is_dirty())
    }

    /// Open `relative`, or focus it when it is already open.
    pub fn open_file(&mut self, relative: &str) -> Result<(), ProjectError> {
        let Some(relative) = normalize_relative(relative) else {
            return Err(ProjectError::UnsafePath(relative.to_owned()));
        };
        if let Some(index) = self.open.iter().position(|doc| doc.relative == relative) {
            self.active = Some(index);
            return Ok(());
        }
        if !self.files.iter().any(|path| path == &relative) {
            return Err(ProjectError::Missing(relative));
        }
        let text = self.read_file(&relative)?;
        self.open.push(Document {
            relative,
            buffer: Buffer::from_text(text),
        });
        self.active = Some(self.open.len() - 1);
        Ok(())
    }

    /// Open the first indexed file. Does nothing when the project is empty.
    pub fn open_first(&mut self) -> Result<(), ProjectError> {
        let Some(path) = self.files.first().cloned() else {
            return Ok(());
        };
        self.open_file(&path)
    }

    pub fn set_active(&mut self, index: usize) -> bool {
        if index < self.open.len() {
            self.active = Some(index);
            true
        } else {
            false
        }
    }

    /// Close an open document. The active document falls on a neighbor.
    pub fn close(&mut self, index: usize) -> bool {
        if index >= self.open.len() {
            return false;
        }
        self.open.remove(index);
        self.active = match self.active {
            Some(active) if active == index => {
                if self.open.is_empty() {
                    None
                } else {
                    Some(index.min(self.open.len() - 1))
                }
            }
            Some(active) if active > index => Some(active - 1),
            other => other,
        };
        true
    }

    /// Write the active buffer. A directory project writes the file; a memory
    /// project updates its stored copy. The buffer is then clean.
    pub fn save_active(&mut self) -> Result<(), ProjectError> {
        let index = self.active.ok_or(ProjectError::NothingOpen)?;
        let relative = self.open[index].relative.clone();
        let text = self.open[index].buffer.text().to_owned();
        if let Some(root) = &self.root {
            let path = root.join(relative_to_path(&relative));
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, text)?;
        } else {
            self.contents.insert(relative, text);
        }
        self.open[index].buffer.mark_clean();
        Ok(())
    }

    fn read_file(&self, relative: &str) -> Result<String, ProjectError> {
        if let Some(root) = &self.root {
            let path = root.join(relative_to_path(relative));
            read_text(&path).map_err(|err| match err {
                ProjectError::Binary(_) => ProjectError::Binary(relative.to_owned()),
                other => other,
            })
        } else {
            self.contents
                .get(relative)
                .cloned()
                .ok_or_else(|| ProjectError::Missing(relative.to_owned()))
        }
    }
}

fn read_text(path: &Path) -> Result<String, ProjectError> {
    let bytes = std::fs::read(path)?;
    if bytes.contains(&0) {
        return Err(ProjectError::Binary(path.display().to_string()));
    }
    String::from_utf8(bytes).map_err(|_| ProjectError::Binary(path.display().to_string()))
}

fn normalize_relative(path: &str) -> Option<String> {
    let path = path.trim().replace('\\', "/");
    let path = path.trim_start_matches("./");
    let path = path.trim_matches('/');
    if path.is_empty() {
        return None;
    }
    let ok = path
        .split('/')
        .all(|part| !part.is_empty() && part != "." && part != "..");
    if ok {
        Some(path.to_owned())
    } else {
        None
    }
}

fn relative_to_path(relative: &str) -> PathBuf {
    relative.split('/').collect()
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn walk(
    root: &Path,
    dir: &Path,
    files: &mut Vec<String>,
    depth: usize,
    truncated: &mut bool,
) -> Result<(), ProjectError> {
    if depth > MAX_DEPTH {
        return Ok(());
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if dir == root => return Err(err.into()),
        Err(_) => return Ok(()),
    };
    let mut entries = entries.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if files.len() >= MAX_FILES {
            *truncated = true;
            break;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if skip_dir(&name) {
                continue;
            }
            walk(root, &entry.path(), files, depth + 1, truncated)?;
            continue;
        }
        if !file_type.is_file() || !is_text_file(&name) {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        if meta.len() > MAX_BYTES {
            continue;
        }
        files.push(relative_path(root, &entry.path()));
    }
    Ok(())
}

fn skip_dir(name: &str) -> bool {
    name.starts_with('.') || SKIP_DIRS.contains(&name)
}

fn is_text_file(name: &str) -> bool {
    if DOTFILES.contains(&name) {
        return true;
    }
    if name.starts_with('.') {
        return false;
    }
    let lower = name.to_ascii_lowercase();
    if TEXT_NAMES.contains(&lower.as_str()) {
        return true;
    }
    match lower.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => TEXT_EXT.contains(&ext),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct DirGuard<'a>(&'a std::path::Path);

    impl Drop for DirGuard<'_> {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.0);
        }
    }

    #[test]
    fn memory_project_saves_and_reopens() {
        let mut project = Project::memory("sample", [("src/main.rs", "fn main() {}\n")]);
        assert!(project.open_file("../etc/passwd").is_err());
        project.open_file("src/main.rs").unwrap();
        assert_eq!(project.active_path(), Some("src/main.rs"));
        project
            .active_mut()
            .unwrap()
            .buffer_mut()
            .set_text("fn main() { println!(\"hi\"); }\n");
        assert!(project.is_dirty("src/main.rs"));
        project.save_active().unwrap();
        assert!(!project.is_dirty("src/main.rs"));
        project.close(0);
        project.open_file("src/main.rs").unwrap();
        assert_eq!(
            project.active().unwrap().buffer().text(),
            "fn main() { println!(\"hi\"); }\n"
        );
    }

    #[test]
    fn directory_index_skips_dependencies_and_binaries() {
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "carboneditor-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let _guard = DirGuard(&dir);
        std::fs::create_dir_all(dir.join("src/util")).unwrap();
        std::fs::create_dir_all(dir.join("node_modules/pkg")).unwrap();
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join("README"), "hello\n").unwrap();
        std::fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dir.join("src/util/mod.rs"), "pub fn util() {}\n").unwrap();
        std::fs::write(
            dir.join("node_modules/pkg/index.js"),
            "module.exports = {}\n",
        )
        .unwrap();
        std::fs::write(dir.join(".git/config"), "[core]\n").unwrap();
        std::fs::write(dir.join("image.png"), [0x89, 0x50, 0x4e, 0x47]).unwrap();

        let mut project = Project::from_dir(&dir).unwrap();
        let mut paths = project.paths().to_vec();
        paths.sort();
        assert_eq!(
            paths,
            vec![
                "README".to_string(),
                "src/main.rs".to_string(),
                "src/util/mod.rs".to_string(),
            ]
        );
        project.open_file("src/main.rs").unwrap();
        assert_eq!(project.active().unwrap().buffer().text(), "fn main() {}\n");
        project
            .active_mut()
            .unwrap()
            .buffer_mut()
            .set_text("fn main() { let x = 1; }\n");
        project.save_active().unwrap();
        assert_eq!(
            std::fs::read_to_string(dir.join("src/main.rs")).unwrap(),
            "fn main() { let x = 1; }\n"
        );
        drop(_guard);
    }
}
