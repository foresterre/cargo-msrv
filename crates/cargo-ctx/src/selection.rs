use crate::Error;
use camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{Metadata, MetadataCommand, Package, WorkspaceDefaultMembers};
use clap_cargo::Workspace;

/// The workspace packages selected
#[derive(Debug)]
pub struct Selection {
    /// root of the workspace.
    pub workspace_root: Utf8PathBuf,
    /// Whether the project has more than one package, or is virtual manifest.
    pub is_workspace: bool,
    /// The packages whivh were selected.
    pub selected: Vec<Package>,
    /// The packages which were not selected.
    pub excluded: Vec<Package>,
}

impl Selection {
    /// Runs `cargo metadata` and selects the packages
    pub fn resolve(manifest_path: &Utf8Path, workspace: &Workspace) -> Result<Self, Error> {
        let metadata = MetadataCommand::new().manifest_path(manifest_path).exec()?;

        Self::from_metadata(metadata, workspace)
    }

    fn from_metadata(mut metadata: Metadata, workspace: &Workspace) -> Result<Self, Error> {
        if metadata.workspace_default_members.is_missing() {
            metadata.workspace_default_members = assume_default_members(&metadata, workspace)?;
        }

        let members = metadata.workspace_packages();

        let root_manifest = metadata.workspace_root.join("Cargo.toml");
        let is_workspace =
            !matches!(members.as_slice(), [only] if only.manifest_path == root_manifest);

        if let Some(unknown) = workspace.package.iter().find(|name| {
            !members
                .iter()
                .any(|member| member.name.as_str() == name.as_str())
        }) {
            return Err(Error::UnknownPackage(unknown.clone()));
        }

        let (selected, excluded) = workspace.partition_packages(&metadata);

        Ok(Self {
            workspace_root: metadata.workspace_root.clone(),
            is_workspace,
            selected: selected.into_iter().cloned().collect(),
            excluded: excluded.into_iter().cloned().collect(),
        })
    }
}

fn assume_default_members(
    metadata: &Metadata,
    workspace: &Workspace,
) -> Result<WorkspaceDefaultMembers, Error> {
    let selects_packages = workspace.workspace
        || workspace.all
        || !workspace.package.is_empty()
        || !workspace.exclude.is_empty();

    if !selects_packages && metadata.workspace_members.len() > 1 {
        return Err(Error::DefaultMembersUnavailable {
            path: metadata.workspace_root.clone(),
        });
    }

    serde_json::to_value(&metadata.workspace_members)
        .and_then(serde_json::from_value)
        .map_err(Error::StoreDefaultMembers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use assert_fs::prelude::*;

    fn package(dir: &TempDir, path: &str, name: &str, extra: &str) {
        dir.child(path)
            .child("Cargo.toml")
            .write_str(&format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n{extra}"
            ))
            .unwrap();
        dir.child(path).child("src/lib.rs").touch().unwrap();
    }

    fn single_package() -> TempDir {
        let dir = TempDir::new().unwrap();
        package(&dir, ".", "single", "");
        dir
    }

    fn root_package_with_members() -> TempDir {
        let dir = TempDir::new().unwrap();
        package(&dir, ".", "root", "[workspace]\nmembers = [\"a\", \"b\"]\n");
        package(&dir, "a", "a", "");
        package(&dir, "b", "b", "");
        dir
    }

    fn metadata(dir: &TempDir) -> Metadata {
        MetadataCommand::new()
            .manifest_path(dir.child("Cargo.toml").path())
            .exec()
            .unwrap()
    }

    fn without_default_members(metadata: &Metadata) -> Metadata {
        let mut json = serde_json::to_value(metadata).unwrap();
        json.as_object_mut()
            .unwrap()
            .remove("workspace_default_members");

        serde_json::from_value(json).unwrap()
    }

    fn workspace(configure: impl FnOnce(&mut Workspace)) -> Workspace {
        let mut workspace = Workspace::default();
        configure(&mut workspace);
        workspace
    }

    fn names(packages: &[Package]) -> Vec<&str> {
        packages
            .iter()
            .map(|package| package.name.as_str())
            .collect()
    }

    #[track_caller]
    fn assert_same_as_cargo(dir: &TempDir, workspace: &Workspace) {
        let metadata = metadata(dir);
        let old = without_default_members(&metadata);
        assert!(old.workspace_default_members.is_missing());

        let expected = Selection::from_metadata(metadata, workspace).unwrap();
        let selection = Selection::from_metadata(old, workspace).unwrap();

        assert_eq!(selection.workspace_root, expected.workspace_root);
        assert_eq!(selection.is_workspace, expected.is_workspace);
        assert_eq!(names(&selection.selected), names(&expected.selected));
        assert_eq!(names(&selection.excluded), names(&expected.excluded));
    }

    #[test]
    fn single_package_without_flags() {
        assert_same_as_cargo(&single_package(), &Workspace::default());
    }

    #[test]
    fn single_package_with_package_flag() {
        let workspace = workspace(|w| w.package = vec!["single".into()]);

        assert_same_as_cargo(&single_package(), &workspace);
    }

    #[test]
    fn workspace_with_workspace_flag() {
        let workspace = workspace(|w| w.workspace = true);

        assert_same_as_cargo(&root_package_with_members(), &workspace);
    }

    #[test]
    fn workspace_with_all_flag() {
        let workspace = workspace(|w| w.all = true);

        assert_same_as_cargo(&root_package_with_members(), &workspace);
    }

    #[test]
    fn workspace_with_package_flag() {
        let workspace = workspace(|w| w.package = vec!["a".into()]);

        assert_same_as_cargo(&root_package_with_members(), &workspace);
    }

    #[test]
    fn workspace_with_exclude_flag() {
        let workspace = workspace(|w| w.exclude = vec!["a".into()]);

        assert_same_as_cargo(&root_package_with_members(), &workspace);
    }

    #[test]
    fn workspace_without_flags() {
        let dir = root_package_with_members();
        let old = without_default_members(&metadata(&dir));

        let error = Selection::from_metadata(old, &Workspace::default()).unwrap_err();

        assert!(
            matches!(&error, Error::DefaultMembersUnavailable { path } if path.as_std_path() == dir.path()),
            "{error:?}"
        );
    }

    #[test]
    fn workspace_without_flags_with_default_members_reported() {
        let dir = root_package_with_members();

        let selection = Selection::from_metadata(metadata(&dir), &Workspace::default()).unwrap();

        assert_eq!(names(&selection.selected), ["root"]);
    }

    #[test]
    fn unknown_package() {
        let dir = root_package_with_members();
        let old = without_default_members(&metadata(&dir));
        let workspace = workspace(|w| w.package = vec!["c".into()]);

        let error = Selection::from_metadata(old, &workspace).unwrap_err();

        assert!(
            matches!(&error, Error::UnknownPackage(name) if name == "c"),
            "{error:?}"
        );
    }
}
