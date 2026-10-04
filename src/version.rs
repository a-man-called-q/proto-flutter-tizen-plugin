use version_spec::{Op, UnresolvedVersionSpec, Version};

// flutter-tizen tags its releases as `<flutter>-tizen.<revision>`, which reads
// as a semver pre-release. Requirements like `3.47` never match pre-releases,
// so the Tizen revision is carried as build metadata instead:
//
//   tag `3.47.1-tizen.1.1.1` <-> version `3.47.1+tizen.1.1.1`
//
// Older releases were tagged with the bare Flutter version (`3.24.1`), and
// those map to themselves.
static TIZEN_PREFIX: &str = "tizen.";

/// Convert a Git tag into a version, or `None` if the tag is not a release.
pub fn tag_to_version(tag: &str) -> Option<Version> {
    let mut version = Version::parse(tag).ok()?;

    if !version.is_semantic() || version.scope.is_some() || version.build.is_some() {
        return None;
    }

    if let Some(pre) = version.prerelease.take() {
        if !pre.starts_with(TIZEN_PREFIX) {
            return None;
        }

        version.build = Some(pre);
    }

    Some(version)
}

/// Convert a version back into the Git tag it was created from.
pub fn version_to_tag(version: &Version) -> String {
    let base = format!("{}.{}.{}", version.major, version.minor, version.patch);

    match version.build.as_ref().or(version.prerelease.as_ref()) {
        Some(revision) => format!("{base}-{revision}"),
        None => base,
    }
}

/// Convert a list of Git tags into a sorted list of installable versions.
/// A bare tag is dropped when the same Flutter version also has Tizen
/// revisions, as an exact `3.44.8` then resolves to its newest revision.
pub fn versions_from_tags<I, T>(tags: I) -> Vec<Version>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let mut versions = tags
        .into_iter()
        .filter_map(|tag| tag_to_version(tag.as_ref()))
        .collect::<Vec<_>>();

    versions.sort();
    versions.dedup();

    let has_revision = |bare: &Version| {
        versions.iter().any(|other| {
            other.build.is_some()
                && other.major == bare.major
                && other.minor == bare.minor
                && other.patch == bare.patch
        })
    };

    versions
        .iter()
        .filter(|version| version.build.is_some() || !has_revision(version))
        .cloned()
        .collect()
}

/// Rewrite a requested version into one that proto can match against the
/// versions above, or `None` when it can be resolved as-is.
pub fn normalize_spec(spec: &UnresolvedVersionSpec) -> Option<UnresolvedVersionSpec> {
    let UnresolvedVersionSpec::Version(version) = spec else {
        return None;
    };

    if !version.is_semantic() || version.scope.is_some() || version.build.is_some() {
        return None;
    }

    match &version.prerelease {
        // The Git tag itself, `3.47.1-tizen.1.1.1`
        Some(pre) if pre.starts_with(TIZEN_PREFIX) => {
            let mut version = version.clone();
            version.build = version.prerelease.take();

            Some(UnresolvedVersionSpec::Version(version))
        }
        Some(_) => None,
        // A bare Flutter version, `3.47.1`, is its newest Tizen revision
        None => Some(UnresolvedVersionSpec::Requirement(
            version.to_requirement(Op::Exact),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use version_spec::MatchesVersion;

    static TAGS: &[&str] = &[
        "2.0.1",
        "3.24.1",
        "3.27.1",
        "3.27.1-tizen.1.0.0",
        "3.27.1-tizen.1.0.1",
        "3.44.8",
        "3.44.8-tizen.1.0.0",
        "3.44.8-tizen.1.1.0",
        "3.47.1-tizen.1.0.0",
        "3.47.1-tizen.1.1.0",
        "3.47.1-tizen.1.1.1",
    ];

    fn strings(versions: &[Version]) -> Vec<String> {
        versions.iter().map(|version| version.to_string()).collect()
    }

    fn resolve(spec: &str) -> Option<String> {
        let versions = versions_from_tags(TAGS);
        let spec = UnresolvedVersionSpec::parse(spec).unwrap();
        let spec = normalize_spec(&spec).unwrap_or(spec);

        let matched = match &spec {
            UnresolvedVersionSpec::Version(version) => {
                versions.iter().filter(|v| *v == version).max()
            }
            UnresolvedVersionSpec::Requirement(req) => {
                versions.iter().filter(|v| req.matches(v)).max()
            }
            UnresolvedVersionSpec::Range(range) => {
                versions.iter().filter(|v| range.matches(v)).max()
            }
            _ => None,
        };

        matched.map(|version| version.to_string())
    }

    #[test]
    fn converts_tags_to_versions() {
        assert_eq!(
            tag_to_version("3.47.1-tizen.1.1.1").unwrap().to_string(),
            "3.47.1+tizen.1.1.1"
        );
        assert_eq!(tag_to_version("3.24.1").unwrap().to_string(), "3.24.1");
    }

    #[test]
    fn ignores_non_release_tags() {
        assert_eq!(tag_to_version("nightly"), None);
        assert_eq!(tag_to_version("3.47.1-beta.1"), None);
        assert_eq!(tag_to_version("3.47.1+tizen.1.0.0"), None);
    }

    #[test]
    fn converts_versions_to_tags() {
        for tag in TAGS {
            assert_eq!(version_to_tag(&tag_to_version(tag).unwrap()), *tag);
        }

        // Tolerate the tag form as well
        assert_eq!(
            version_to_tag(&Version::parse("3.47.1-tizen.1.1.1").unwrap()),
            "3.47.1-tizen.1.1.1"
        );
    }

    #[test]
    fn sorts_versions_and_drops_shadowed_bare_tags() {
        assert_eq!(
            strings(&versions_from_tags(TAGS.iter().rev())),
            [
                "2.0.1",
                "3.24.1",
                "3.27.1+tizen.1.0.0",
                "3.27.1+tizen.1.0.1",
                "3.44.8+tizen.1.0.0",
                "3.44.8+tizen.1.1.0",
                "3.47.1+tizen.1.0.0",
                "3.47.1+tizen.1.1.0",
                "3.47.1+tizen.1.1.1",
            ]
        );
    }

    #[test]
    fn orders_revisions_numerically() {
        let versions = versions_from_tags(["3.47.1-tizen.1.10.0", "3.47.1-tizen.1.9.0"]);

        assert_eq!(versions.last().unwrap().to_string(), "3.47.1+tizen.1.10.0");
    }

    #[test]
    fn resolves_partial_versions_to_newest_revision() {
        assert_eq!(resolve("3.47").unwrap(), "3.47.1+tizen.1.1.1");
        assert_eq!(resolve("3.44").unwrap(), "3.44.8+tizen.1.1.0");
        assert_eq!(resolve("3").unwrap(), "3.47.1+tizen.1.1.1");
        assert_eq!(resolve("^3.27").unwrap(), "3.47.1+tizen.1.1.1");
        assert_eq!(resolve("~3.27").unwrap(), "3.27.1+tizen.1.0.1");
        assert_eq!(resolve(">=3.24 <3.44").unwrap(), "3.27.1+tizen.1.0.1");
    }

    #[test]
    fn resolves_bare_versions() {
        assert_eq!(resolve("3.47.1").unwrap(), "3.47.1+tizen.1.1.1");
        assert_eq!(resolve("3.44.8").unwrap(), "3.44.8+tizen.1.1.0");
        assert_eq!(resolve("3.24.1").unwrap(), "3.24.1");
        assert_eq!(resolve("99.99.99"), None);
    }

    #[test]
    fn resolves_exact_revisions() {
        assert_eq!(resolve("3.47.1-tizen.1.1.0").unwrap(), "3.47.1+tizen.1.1.0");
        assert_eq!(resolve("3.47.1+tizen.1.0.0").unwrap(), "3.47.1+tizen.1.0.0");
        assert_eq!(resolve("3.47.1-tizen.9.9.9"), None);
    }

    #[test]
    fn leaves_other_specs_alone() {
        for spec in ["latest", "3.47", "^3", "3.47.1+tizen.1.0.0", "3.47.1-rc.1"] {
            assert_eq!(
                normalize_spec(&UnresolvedVersionSpec::parse(spec).unwrap()),
                None,
                "{spec}"
            );
        }
    }
}
