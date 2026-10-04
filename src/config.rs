#[derive(Debug, schematic::Schematic, serde::Deserialize, serde::Serialize)]
#[serde(default, deny_unknown_fields, rename_all = "kebab-case")]
pub struct FlutterTizenToolConfig {
    /// Git repository to load tags from and clone.
    pub repo_url: String,

    /// Skip cloning the Flutter SDK and compiling the tool snapshot during
    /// install. They will then happen on the first `flutter-tizen` run.
    pub skip_bootstrap: bool,
}

impl Default for FlutterTizenToolConfig {
    fn default() -> Self {
        Self {
            repo_url: "https://github.com/flutter-tizen/flutter-tizen.git".into(),
            skip_bootstrap: false,
        }
    }
}
