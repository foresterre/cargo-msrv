use cargo_metadata::Package;
use cargo_util_schemas::core::PackageIdSpec;

pub trait PackageIdSpecQuery {
    fn matches(&self, package: &Package) -> bool;
}

impl PackageIdSpecQuery for PackageIdSpec {
    fn matches(&self, package: &Package) -> bool {
        if self.name() != package.name.as_str() {
            return false;
        }

        if let Some(version) = self.partial_version()
            && !version.matches(&package.version)
        {
            return false;
        }

        if self.url().is_none() && self.kind().is_none() {
            return true;
        }

        let Some(package_spec) = package_id_spec(package) else {
            return false;
        };

        self.url().is_none_or(|url| package_spec.url() == Some(url))
            && self
                .kind()
                .is_none_or(|kind| package_spec.kind() == Some(kind))
    }
}

fn package_id_spec(package: &Package) -> Option<PackageIdSpec> {
    PackageIdSpec::parse(&package.id.repr).ok().or_else(|| {
        let source = package.source.as_ref()?;
        let url = source
            .repr
            .split_once('#')
            .map_or(source.repr.as_str(), |(url, _)| url);
        let spec = format!("{url}#{}@{}", package.name, package.version);

        PackageIdSpec::parse(&spec).ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(id: &str, source: Option<&str>, name: &str, version: &str) -> Package {
        serde_json::from_value(serde_json::json!({
            "name": name,
            "version": version,
            "id": id,
            "source": source,
            "dependencies": [],
            "targets": [],
            "features": {},
            "manifest_path": "/path/to/Cargo.toml",
        }))
        .unwrap()
    }

    fn serde() -> Package {
        package(
            "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.228",
            Some("registry+https://github.com/rust-lang/crates.io-index"),
            "serde",
            "1.0.228",
        )
    }

    fn matches(spec: &str, package: &Package) -> bool {
        PackageIdSpec::parse(spec).unwrap().matches(package)
    }

    #[yare::parameterized(
        name = { "serde" },
        major = { "serde@1" },
        minor = { "serde@1.0" },
        full_version = { "serde@1.0.228" },
        url = { "https://github.com/rust-lang/crates.io-index#serde" },
        kind_and_url = { "registry+https://github.com/rust-lang/crates.io-index#serde@1.0.228" },
    )]
    fn matches_package(spec: &str) {
        assert!(matches(spec, &serde()));
    }

    #[yare::parameterized(
        other_name = { "serde_json" },
        other_version = { "serde@2" },
        other_url = { "https://example.com/index#serde" },
        other_kind = { "git+https://github.com/rust-lang/crates.io-index#serde" },
    )]
    fn does_not_match_package(spec: &str) {
        assert!(!matches(spec, &serde()));
    }

    #[test]
    fn matches_git_package() {
        let package = package(
            "git+https://github.com/foo/bar?branch=main#bar@0.1.0",
            Some("git+https://github.com/foo/bar?branch=main#0123456789abcdef"),
            "bar",
            "0.1.0",
        );

        assert!(matches(
            "git+https://github.com/foo/bar?branch=main#bar",
            &package
        ));
        assert!(!matches(
            "git+https://github.com/foo/bar?branch=dev#bar",
            &package
        ));
    }

    #[test]
    fn matches_path_package() {
        let package = package("path+file:///path/to/foo#0.1.0", None, "foo", "0.1.0");

        assert!(matches("path+file:///path/to/foo#foo@0.1.0", &package));
        assert!(matches("file:///path/to/foo", &package));
        assert!(!matches("file:///path/to/bar#foo", &package));
    }

    #[test]
    fn matches_source_with_old_package_id_format() {
        let package = package(
            "bar 0.1.0 (git+https://github.com/foo/bar?branch=main#0123456789abcdef)",
            Some("git+https://github.com/foo/bar?branch=main#0123456789abcdef"),
            "bar",
            "0.1.0",
        );

        assert!(matches(
            "git+https://github.com/foo/bar?branch=main#bar",
            &package
        ));
    }

    #[test]
    fn does_not_match_url_without_known_source() {
        let package = package("foo 0.1.0 (path+file:///path/to/foo)", None, "foo", "0.1.0");

        assert!(matches("foo@0.1.0", &package));
        assert!(!matches("file:///path/to/foo", &package));
    }
}
