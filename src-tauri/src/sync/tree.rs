//! Pure transformation: `core_course_get_contents` → files to download and external links.
//!
//! Local layout, relative to the course folder (mirrors the v1 so its folders can be adopted):
//! `<section>/<subsection>/<module>/<sub/dir>/<file>`, where
//! - the "Materiali" section is flattened by default (as in the v1);
//! - a `resource` with a single file is saved as `<section>/<module name>.<ext>`;
//! - every other module with files gets its own folder (`folder`, `page`, unknown types …):
//!   no module type is excluded a priori, which is how the v1 lost the `page` files (§5).

use std::collections::{HashMap, HashSet};

use crate::moodle::types::{CourseModule, CourseSection, ModuleContent};
use crate::sync::extract::{LinkedFile, linked_files, webservice_file};
use crate::sync::paths::{ensure_extension, extension, sanitize_component};

/// Component name of Moodle 4.5 delegated sections (subsections).
const SUBSECTION_COMPONENT: &str = "mod_subsection";
const SUBSECTION_MODNAME: &str = "subsection";
const RESOURCE_MODNAME: &str = "resource";
const URL_MODNAME: &str = "url";
const LABEL_MODNAME: &str = "label";
/// Module id used for files linked from a section summary (they belong to no module).
const SECTION_LINK_MODULE_ID: u64 = 0;
/// Modules whose `index.html` is the rendered page body, not a downloadable file.
const HTML_BODY_MODNAMES: &[&str] = &["page", "book"];
const HTML_BODY_FILENAME: &str = "index.html";
/// Section name fragment flattened when [`TreeOptions::flatten_materials`] is on.
const MATERIALS_MARKER: &str = "material";
/// Moodle allows one level of subsections; the limit only guards against malformed data.
const MAX_SUBSECTION_DEPTH: usize = 3;

#[derive(Debug, Clone, Copy)]
pub struct TreeOptions {
    /// Do not create a folder for the "Materiali" section.
    pub flatten_materials: bool,
}

impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            flatten_materials: true,
        }
    }
}

/// A file to download, with its sanitized and de-duplicated local path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFile {
    pub module_id: u64,
    /// Stable identity of the remote file: `<module id>:<filepath><filename>`.
    pub remote_key: String,
    /// Path components relative to the course folder; the last one is the file name.
    pub local_path: Vec<String>,
    pub fileurl: String,
    pub filesize: u64,
    pub timemodified: i64,
}

/// Content that cannot be downloaded (OneDrive, Kaggle, recordings, …): listed for the user.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalLink {
    pub module_id: u64,
    pub section: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CourseTree {
    pub files: Vec<RemoteFile>,
    pub external_links: Vec<ExternalLink>,
}

/// Builds the download tree of a course.
pub fn build_tree(sections: &[CourseSection], options: TreeOptions) -> CourseTree {
    let delegated = sections
        .iter()
        .filter(|s| is_delegated(s))
        .filter_map(|s| s.itemid.map(|id| (id, s)))
        .collect();
    let mut walker = Walker {
        options,
        delegated,
        drafts: Vec::new(),
        links: Vec::new(),
    };
    for section in sections.iter().filter(|s| !is_delegated(s)) {
        let prefix = walker.section_prefix(section);
        walker.walk_section(section, &prefix, 0);
    }
    CourseTree {
        files: layout(&walker.drafts),
        external_links: walker.links,
    }
}

/// A file before sanitization and de-duplication of its local path.
struct Draft {
    module_id: u64,
    remote_key: String,
    components: Vec<String>,
    fileurl: String,
    filesize: u64,
    timemodified: i64,
    /// Linked from HTML or a `url` module: size and date unknown, lower naming priority.
    linked: bool,
}

struct Walker<'a> {
    options: TreeOptions,
    /// Delegated (subsection) sections, by the instance id of their `subsection` module.
    delegated: HashMap<u64, &'a CourseSection>,
    drafts: Vec<Draft>,
    links: Vec<ExternalLink>,
}

impl Walker<'_> {
    fn section_prefix(&self, section: &CourseSection) -> Vec<String> {
        let name = section_name(section);
        let is_materials = name.to_lowercase().contains(MATERIALS_MARKER);
        if self.options.flatten_materials && is_materials {
            Vec::new()
        } else {
            vec![name]
        }
    }

    fn walk_section(&mut self, section: &CourseSection, prefix: &[String], depth: usize) {
        if section.uservisible == Some(false) {
            return;
        }
        let name = section_name(section);
        let summary_links = linked_files(section.summary.as_deref().unwrap_or_default());
        self.drafts.extend(summary_links.iter().map(|file| {
            let key = format!("section{}:link:{}", section.id, without_query(&file.url));
            link_draft(
                SECTION_LINK_MODULE_ID,
                key,
                file,
                prefix_with(prefix, &file.filename),
            )
        }));
        for module in section
            .modules
            .iter()
            .filter(|m| m.uservisible != Some(false))
        {
            match module.modname.as_str() {
                SUBSECTION_MODNAME => self.walk_subsection(module, prefix, depth),
                URL_MODNAME => self.visit_url(module, &name, prefix),
                LABEL_MODNAME => self.drafts.extend(label_drafts(module, prefix)),
                _ => {
                    self.drafts.extend(module_drafts(module, prefix));
                    self.drafts.extend(intro_drafts(module, prefix));
                }
            }
        }
    }

    /// A `url` module pointing to a WeBeep file is a download; anything else is external.
    fn visit_url(&mut self, module: &CourseModule, section: &str, prefix: &[String]) {
        for content in module.contents.iter().filter(|c| c.kind == "url") {
            match webservice_file(&content.fileurl) {
                Some(file) => {
                    let filename = resource_name(&module.name, &file.filename);
                    let key = module_link_key(module.id, &file);
                    let components = prefix_with(prefix, &filename);
                    self.drafts
                        .push(link_draft(module.id, key, &file, components));
                }
                None if !content.fileurl.is_empty() => self.links.push(ExternalLink {
                    module_id: module.id,
                    section: section.to_owned(),
                    name: module.name.clone(),
                    url: content.fileurl.clone(),
                }),
                None => {}
            }
        }
    }

    fn walk_subsection(&mut self, module: &CourseModule, prefix: &[String], depth: usize) {
        if depth >= MAX_SUBSECTION_DEPTH {
            return;
        }
        let Some(section) = module
            .instance
            .and_then(|i| self.delegated.get(&i).copied())
        else {
            return;
        };
        let nested = [prefix, std::slice::from_ref(&module.name)].concat();
        self.walk_section(section, &nested, depth + 1);
    }
}

fn is_delegated(section: &CourseSection) -> bool {
    section.component.as_deref() == Some(SUBSECTION_COMPONENT)
}

fn section_name(section: &CourseSection) -> String {
    if section.name.trim().is_empty() {
        format!("Sezione {}", section.section)
    } else {
        section.name.clone()
    }
}

fn module_drafts(module: &CourseModule, prefix: &[String]) -> Vec<Draft> {
    let files: Vec<&ModuleContent> = module
        .contents
        .iter()
        .filter(|c| c.kind == "file" && !is_html_body(module, c))
        .collect();
    let single_resource = module.modname == RESOURCE_MODNAME && files.len() == 1;
    files
        .into_iter()
        .map(|content| {
            let filename = ensure_extension(&content.filename, content.mimetype.as_deref());
            let relative = if single_resource {
                vec![resource_name(&module.name, &filename)]
            } else {
                let mut parts = vec![module_folder(module, &filename)];
                parts.extend(subdirs(content.filepath.as_deref()));
                parts.push(filename);
                parts
            };
            draft(module, content, [prefix, &relative].concat())
        })
        .collect()
}

fn draft(module: &CourseModule, content: &ModuleContent, components: Vec<String>) -> Draft {
    let filepath = content.filepath.as_deref().unwrap_or("/");
    Draft {
        module_id: module.id,
        remote_key: format!("{}:{filepath}{}", module.id, content.filename),
        components,
        fileurl: content.fileurl.clone(),
        filesize: content.filesize,
        timemodified: content.timemodified.unwrap_or_default(),
        linked: false,
    }
}

/// Labels are shown inline in the section: their linked files go in the section folder.
fn label_drafts(module: &CourseModule, prefix: &[String]) -> Vec<Draft> {
    linked_files(module.description.as_deref().unwrap_or_default())
        .iter()
        .map(|file| {
            let key = module_link_key(module.id, file);
            link_draft(module.id, key, file, prefix_with(prefix, &file.filename))
        })
        .collect()
}

/// Files linked in a module intro go in the module folder, unless already in its contents.
fn intro_drafts(module: &CourseModule, prefix: &[String]) -> Vec<Draft> {
    let listed: HashSet<&str> = module
        .contents
        .iter()
        .map(|c| without_query(&c.fileurl))
        .collect();
    linked_files(module.description.as_deref().unwrap_or_default())
        .iter()
        .filter(|file| !listed.contains(without_query(&file.url)))
        .map(|file| {
            let folder = module_folder(module, &file.filename);
            let components = [prefix, &[folder, file.filename.clone()]].concat();
            link_draft(
                module.id,
                module_link_key(module.id, file),
                file,
                components,
            )
        })
        .collect()
}

fn link_draft(
    module_id: u64,
    remote_key: String,
    file: &LinkedFile,
    components: Vec<String>,
) -> Draft {
    Draft {
        module_id,
        remote_key,
        components,
        fileurl: file.url.clone(),
        filesize: 0,
        timemodified: 0,
        linked: true,
    }
}

fn module_link_key(module_id: u64, file: &LinkedFile) -> String {
    format!("{module_id}:link:{}", without_query(&file.url))
}

fn without_query(url: &str) -> &str {
    url.split(['?', '#']).next().unwrap_or(url)
}

fn prefix_with(prefix: &[String], name: &str) -> Vec<String> {
    [prefix, &[name.to_owned()]].concat()
}

fn is_html_body(module: &CourseModule, content: &ModuleContent) -> bool {
    HTML_BODY_MODNAMES.contains(&module.modname.as_str()) && content.filename == HTML_BODY_FILENAME
}

fn module_folder(module: &CourseModule, filename: &str) -> String {
    if module.name.trim().is_empty() {
        filename.to_owned()
    } else {
        module.name.clone()
    }
}

/// A single-file resource is shown on WeBeep with the module name: keep it, with the
/// file's extension.
fn resource_name(module_name: &str, filename: &str) -> String {
    if module_name.trim().is_empty() {
        return filename.to_owned();
    }
    match extension(filename) {
        Some(ext)
            if !module_name
                .to_lowercase()
                .ends_with(&format!(".{}", ext.to_lowercase())) =>
        {
            format!("{module_name}.{ext}")
        }
        _ => module_name.to_owned(),
    }
}

fn subdirs(filepath: Option<&str>) -> Vec<String> {
    filepath
        .unwrap_or("/")
        .split('/')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Sanitizes every path and resolves collisions (case-insensitive, as on Windows and macOS).
/// Files listed by Moodle win over linked ones; then the lowest module id keeps the plain
/// name and the others get ` (2)`, ` (3)` … Module ids only grow over time, so existing
/// files keep their names when new ones appear.
fn layout(drafts: &[Draft]) -> Vec<RemoteFile> {
    let mut priority: Vec<usize> = (0..drafts.len()).collect();
    priority.sort_by(|&a, &b| {
        let key = |d: &Draft| (d.linked, d.module_id, d.remote_key.clone());
        key(&drafts[a]).cmp(&key(&drafts[b]))
    });
    let mut taken = HashSet::new();
    let mut paths = vec![Vec::new(); drafts.len()];
    for index in priority {
        let sanitized = drafts[index]
            .components
            .iter()
            .map(|c| sanitize_component(c))
            .collect();
        paths[index] = unique_path(sanitized, &mut taken);
    }
    drafts
        .iter()
        .zip(paths)
        .map(|(draft, local_path)| RemoteFile {
            module_id: draft.module_id,
            remote_key: draft.remote_key.clone(),
            local_path,
            fileurl: draft.fileurl.clone(),
            filesize: draft.filesize,
            timemodified: draft.timemodified,
        })
        .collect()
}

fn unique_path(path: Vec<String>, taken: &mut HashSet<String>) -> Vec<String> {
    let mut candidate = path.clone();
    let mut counter = 2;
    let Some(last) = path.len().checked_sub(1) else {
        return candidate;
    };
    while !taken.insert(candidate.join("/").to_lowercase()) {
        candidate[last] = with_suffix(&path[last], counter);
        counter += 1;
    }
    candidate
}

fn with_suffix(name: &str, counter: u32) -> String {
    match extension(name) {
        Some(ext) => {
            let stem = &name[..name.len() - ext.len() - 1];
            format!("{stem} ({counter}).{ext}")
        }
        None => format!("{name} ({counter})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::fixture;
    use serde_json::json;

    fn tree_of(name: &str, options: TreeOptions) -> CourseTree {
        let sections: Vec<CourseSection> = serde_json::from_str(&fixture(name)).unwrap();
        build_tree(&sections, options)
    }

    fn paths(tree: &CourseTree) -> Vec<String> {
        tree.files.iter().map(|f| f.local_path.join("/")).collect()
    }

    fn sections_from(value: serde_json::Value) -> Vec<CourseSection> {
        serde_json::from_value(value).unwrap()
    }

    fn file(name: &str, path: &str) -> serde_json::Value {
        json!({"type": "file", "filename": name, "filepath": path, "filesize": 10,
               "fileurl": format!("https://webeep.polimi.it/webservice/pluginfile.php/1/{name}"),
               "timemodified": 100, "mimetype": "application/pdf"})
    }

    // --- Real fixtures -------------------------------------------------------------------

    #[test]
    fn data_mining_keeps_every_nested_folder_file() {
        let tree = tree_of("contents/data_mining.json", TreeOptions::default());
        let paths = paths(&tree);
        assert_eq!(paths.len(), 80);
        assert!(paths.contains(&"Syllabus/Slides/DM2026-2027-01-DataMining.pdf".to_owned()));
        assert!(paths.contains(&"Exams/2017-2018/DMTM20170421.pdf".to_owned()));
    }

    #[test]
    fn data_mining_zips_without_extension_get_one() {
        let tree = tree_of("contents/data_mining.json", TreeOptions::default());
        let paths = paths(&tree);
        assert!(
            paths.contains(
                &"Syllabus/Notebooks/Optional Notebooks/06 Classification.zip".to_owned()
            )
        );
        assert!(!paths.iter().any(|p| p.ends_with("/06 Classification")));
    }

    #[test]
    fn data_mining_recordings_are_external_links() {
        let tree = tree_of("contents/data_mining.json", TreeOptions::default());
        assert_eq!(tree.external_links.len(), 1);
        let link = &tree.external_links[0];
        assert_eq!(link.section, "Registrazioni");
        assert_eq!(link.name, "Archivio registrazioni");
        assert!(link.url.starts_with("https://aunicalogin.polimi.it/"));
    }

    #[test]
    fn materials_section_becomes_a_folder_when_not_flattened() {
        let options = TreeOptions {
            flatten_materials: false,
        };
        let tree = tree_of("contents/data_mining.json", options);
        assert!(tree.files.iter().all(|f| f.local_path[0] == "Materiali"));
    }

    #[test]
    fn page_files_are_downloaded_without_index_html() {
        let tree = tree_of(
            "contents/data_information_quality.json",
            TreeOptions::default(),
        );
        let paths = paths(&tree);
        assert_eq!(paths.len(), 21);
        assert!(paths.contains(&"Course Material/01 intro.pdf".to_owned()));
        assert!(!paths.iter().any(|p| p.ends_with("index.html")));
    }

    #[test]
    fn single_file_resources_are_named_after_the_module() {
        let tree = tree_of(
            "contents/technologies_information_systems.json",
            TreeOptions::default(),
        );
        let paths = paths(&tree);
        assert!(paths.contains(&"Introduzione/Course Information.pdf".to_owned()));
        assert!(paths.contains(
            &"Introduzione/Course Schedule Updated on October 9. Classroom Change.pdf".to_owned()
        ));
    }

    #[test]
    fn page_inside_introduction_keeps_section_folder() {
        let tree = tree_of(
            "contents/formal_languages_compilers.json",
            TreeOptions::default(),
        );
        let paths = paths(&tree);
        assert!(
            paths.contains(
                &"Introduzione/Tutoring Sessions (calendar and videos)/FLC_LAB_2021-06-14.zip"
                    .to_owned()
            )
        );
    }

    #[test]
    fn course_without_material_has_no_files() {
        let tree = tree_of("contents/machine_learning.json", TreeOptions::default());
        assert!(tree.files.is_empty());
        assert_eq!(tree.external_links.len(), 1);
    }

    #[test]
    fn remote_keys_are_unique_and_stable() {
        let tree = tree_of(
            "contents/formal_languages_compilers.json",
            TreeOptions::default(),
        );
        let keys: HashSet<_> = tree.files.iter().map(|f| f.remote_key.as_str()).collect();
        assert_eq!(keys.len(), tree.files.len());
        let again = tree_of(
            "contents/formal_languages_compilers.json",
            TreeOptions::default(),
        );
        assert_eq!(tree, again);
    }

    #[test]
    fn local_paths_are_unique_ignoring_case() {
        for name in [
            "contents/data_mining.json",
            "contents/formal_languages_compilers.json",
            "contents/technologies_information_systems.json",
        ] {
            let tree = tree_of(name, TreeOptions::default());
            let unique: HashSet<_> = paths(&tree).iter().map(|p| p.to_lowercase()).collect();
            assert_eq!(unique.len(), tree.files.len(), "{name}");
        }
    }

    // --- Synthetic cases -----------------------------------------------------------------

    #[test]
    fn subsection_content_is_nested_under_its_module() {
        let sections = sections_from(json!([
            {"id": 1, "name": "Week plan", "section": 1, "modules": [
                {"id": 10, "instance": 7, "name": "Week 1", "modname": "subsection", "contents": []}
            ]},
            {"id": 2, "name": "Week 1", "section": 2, "component": "mod_subsection", "itemid": 7,
             "modules": [
                {"id": 11, "name": "Notes", "modname": "folder",
                 "contents": [file("a.pdf", "/")]}
            ]}
        ]));
        let tree = build_tree(&sections, TreeOptions::default());
        assert_eq!(paths(&tree), vec!["Week plan/Week 1/Notes/a.pdf"]);
    }

    #[test]
    fn colliding_names_get_stable_numbered_suffixes() {
        let sections = sections_from(json!([
            {"id": 1, "name": "Lezioni", "modules": [
                {"id": 30, "name": "Slides", "modname": "resource", "contents": [file("b.pdf", "/")]},
                {"id": 20, "name": "slides", "modname": "resource", "contents": [file("a.pdf", "/")]}
            ]}
        ]));
        let tree = build_tree(&sections, TreeOptions::default());
        let by_module: HashMap<u64, String> = tree
            .files
            .iter()
            .map(|f| (f.module_id, f.local_path.join("/")))
            .collect();
        assert_eq!(by_module[&20], "Lezioni/slides.pdf");
        assert_eq!(by_module[&30], "Lezioni/Slides (2).pdf");
    }

    #[test]
    fn hidden_modules_and_activities_without_files_are_skipped() {
        let sections = sections_from(json!([
            {"id": 1, "name": "Lezioni", "modules": [
                {"id": 1, "name": "Hidden", "modname": "folder", "uservisible": false,
                 "contents": [file("x.pdf", "/")]},
                {"id": 2, "name": "Quiz", "modname": "quiz"},
                {"id": 3, "name": "Forum", "modname": "forum", "contents": []}
            ]}
        ]));
        assert!(
            build_tree(&sections, TreeOptions::default())
                .files
                .is_empty()
        );
    }

    #[test]
    fn unknown_module_types_with_files_are_kept() {
        let sections = sections_from(json!([
            {"id": 1, "name": "Lezioni", "modules": [
                {"id": 5, "name": "Dispense", "modname": "book",
                 "contents": [file("index.html", "/1/"), file("cap1.pdf", "/1/")]}
            ]}
        ]));
        let tree = build_tree(&sections, TreeOptions::default());
        assert_eq!(paths(&tree), vec!["Lezioni/Dispense/1/cap1.pdf"]);
    }

    #[test]
    fn names_are_sanitized_for_windows() {
        let sections = sections_from(json!([
            {"id": 1, "name": "Lab: parte 1", "modules": [
                {"id": 5, "name": "Esercizi?", "modname": "folder",
                 "contents": [file("sol*.pdf", "/a|b/")]}
            ]}
        ]));
        let tree = build_tree(&sections, TreeOptions::default());
        assert_eq!(paths(&tree), vec!["Lab_ parte 1/Esercizi_/a_b/sol_.pdf"]);
    }

    #[test]
    fn unnamed_section_gets_a_numbered_folder() {
        let sections = sections_from(json!([
            {"id": 1, "name": "", "section": 4, "modules": [
                {"id": 5, "name": "Doc", "modname": "resource", "contents": [file("d.pdf", "/")]}
            ]}
        ]));
        let tree = build_tree(&sections, TreeOptions::default());
        assert_eq!(paths(&tree), vec!["Sezione 4/Doc.pdf"]);
    }

    // --- Links extracted from HTML and url modules ----------------------------------------

    const SITE_FILE: &str = "https://webeep.polimi.it/pluginfile.php/9/mod_label/intro";
    const WS_FILE: &str = "https://webeep.polimi.it/webservice/pluginfile.php/9/mod_label/intro";

    fn anchor(name: &str) -> String {
        format!(r#"<p><a href="{SITE_FILE}/{name}">{name}</a></p>"#)
    }

    #[test]
    fn label_links_are_saved_in_the_section_folder() {
        let sections = sections_from(json!([
            {"id": 1, "name": "Lezioni", "modules": [
                {"id": 4, "name": "", "modname": "label", "description": anchor("Esercizi%201.pdf")}
            ]}
        ]));
        let tree = build_tree(&sections, TreeOptions::default());
        assert_eq!(paths(&tree), vec!["Lezioni/Esercizi 1.pdf"]);
        let file = &tree.files[0];
        assert_eq!(file.fileurl, format!("{WS_FILE}/Esercizi%201.pdf"));
        assert_eq!(file.filesize, 0, "size of linked files is unknown");
        assert_eq!(file.module_id, 4);
    }

    #[test]
    fn intro_links_go_in_the_module_folder_without_duplicates() {
        let already_listed = json!({"type": "file", "filename": "b.pdf", "filepath": "/",
            "filesize": 5, "fileurl": format!("{WS_FILE}/b.pdf"), "mimetype": "application/pdf"});
        let sections = sections_from(json!([
            {"id": 1, "name": "Lezioni", "modules": [
                {"id": 6, "name": "Cartella", "modname": "folder",
                 "description": format!("{}{}", anchor("a.pdf"), anchor("b.pdf")),
                 "contents": [already_listed]}
            ]}
        ]));
        let tree = build_tree(&sections, TreeOptions::default());
        assert_eq!(
            paths(&tree),
            vec!["Lezioni/Cartella/b.pdf", "Lezioni/Cartella/a.pdf"]
        );
    }

    #[test]
    fn section_summary_links_are_saved_in_the_section_folder() {
        let sections = sections_from(json!([
            {"id": 3, "name": "Esami", "summary": anchor("temi.zip"), "modules": []}
        ]));
        let tree = build_tree(&sections, TreeOptions::default());
        assert_eq!(paths(&tree), vec!["Esami/temi.zip"]);
    }

    #[test]
    fn internal_url_modules_are_downloaded_not_listed() {
        let sections = sections_from(json!([
            {"id": 1, "name": "Lezioni", "modules": [
                {"id": 8, "name": "Slide lezione 1", "modname": "url", "contents": [
                    {"type": "url", "filename": "Slide lezione 1",
                     "fileurl": format!("{SITE_FILE}/l1.pdf")}
                ]}
            ]}
        ]));
        let tree = build_tree(&sections, TreeOptions::default());
        assert_eq!(paths(&tree), vec!["Lezioni/Slide lezione 1.pdf"]);
        assert!(tree.external_links.is_empty());
    }

    #[test]
    fn real_files_win_name_collisions_against_linked_files() {
        let sections = sections_from(json!([
            {"id": 1, "name": "Lezioni", "modules": [
                {"id": 1, "name": "", "modname": "label", "description": anchor("a.pdf")},
                {"id": 2, "name": "a", "modname": "resource", "contents": [file("x.pdf", "/")]}
            ]}
        ]));
        let tree = build_tree(&sections, TreeOptions::default());
        let by_module: HashMap<u64, String> = tree
            .files
            .iter()
            .map(|f| (f.module_id, f.local_path.join("/")))
            .collect();
        assert_eq!(by_module[&2], "Lezioni/a.pdf");
        assert_eq!(by_module[&1], "Lezioni/a (2).pdf");
    }
}
