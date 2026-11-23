//! File list for a [`crate::Project`].

use std::collections::BTreeMap;

use egui::{CollapsingHeader, RichText, ScrollArea, Ui};

use crate::theme::Theme;
use crate::Project;

/// Project files as a folder tree. [`Self::show`] returns the path of a file
/// clicked this frame.
pub struct ProjectTree<'a> {
    project: &'a Project,
    filter: &'a str,
}

impl<'a> ProjectTree<'a> {
    pub fn new(project: &'a Project) -> Self {
        Self {
            project,
            filter: "",
        }
    }

    /// Case-insensitive substring matched against the project-relative path.
    /// An empty filter shows every file.
    pub fn filter(mut self, filter: &'a str) -> Self {
        self.filter = filter;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Option<String> {
        let theme = Theme::of(ui);
        let root = build_tree(self.project.paths(), self.filter);
        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            if root.dirs.is_empty() && root.files.is_empty() {
                ui.label(
                    RichText::new(if self.filter.trim().is_empty() {
                        "No files"
                    } else {
                        "No matching files"
                    })
                    .color(theme.text_weak),
                );
                return None;
            }
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| show_children(ui, &root, self.project, 0))
                .inner
        })
        .inner
    }
}

struct Node {
    name: String,
    path: String,
    dirs: BTreeMap<String, Node>,
    files: Vec<String>,
}

impl Node {
    fn dir(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            dirs: BTreeMap::new(),
            files: Vec::new(),
        }
    }
}

fn build_tree(paths: &[String], filter: &str) -> Node {
    let needle = filter.trim().to_ascii_lowercase();
    let mut root = Node::dir("", "");
    for file in paths {
        if !needle.is_empty() && !file.to_ascii_lowercase().contains(&needle) {
            continue;
        }
        let mut parts = file.split('/').collect::<Vec<_>>();
        let Some(name) = parts.pop() else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let mut node = &mut root;
        let mut acc = String::new();
        for part in parts {
            if !acc.is_empty() {
                acc.push('/');
            }
            acc.push_str(part);
            node = node
                .dirs
                .entry(part.to_owned())
                .or_insert_with(|| Node::dir(part, acc.clone()));
        }
        node.files.push(file.clone());
    }
    root
}

fn show_children(ui: &mut Ui, node: &Node, project: &Project, depth: usize) -> Option<String> {
    let mut clicked = None;
    for child in node.dirs.values() {
        let header = CollapsingHeader::new(RichText::new(&child.name))
            .default_open(depth < 1)
            .id_salt(&child.path)
            .show(ui, |ui| show_children(ui, child, project, depth + 1));
        if let Some(path) = header.body_returned.flatten() {
            clicked = Some(path);
        }
    }
    let active = project.active_path();
    let theme = Theme::of(ui);
    for file in &node.files {
        let name = file.rsplit('/').next().unwrap_or(file);
        let mut label = name.to_owned();
        if project.is_dirty(file) {
            label.push_str("  •");
        }
        let selected = active == Some(file.as_str());
        let response = ui.push_id(file, |ui| {
            let text = if selected {
                RichText::new(label).color(theme.text)
            } else {
                RichText::new(label).color(theme.text_weak)
            };
            ui.selectable_label(selected, text)
        });
        if response.inner.clicked() {
            clicked = Some(file.clone());
        }
    }
    clicked
}
