use proto_pdk_test_utils::*;

mod flutter_tizen_tool {
    use super::*;

    // Only old Flutter versions, as they no longer receive Tizen revisions
    generate_resolve_versions_tests!("flutter-tizen-test", {
        "3.38" => "3.38.8+tizen.1.0.2",
        "~3.35" => "3.35.3+tizen.1.3.0",
        "3.41.9" => "3.41.9+tizen.1.0.1",
        "3.24.1" => "3.24.1",
        "3.44.4-tizen.1.0.0" => "3.44.4+tizen.1.0.0",
        "3.44.1+tizen.1.0.0" => "3.44.1+tizen.1.0.0",
    });

    #[tokio::test(flavor = "multi_thread")]
    async fn loads_versions_from_git_tags() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("flutter-tizen-test").await;

        let output = plugin.load_versions(LoadVersionsInput::default()).await;

        assert!(
            output
                .versions
                .contains(&VersionSpec::parse("3.47.1+tizen.1.0.0").unwrap())
        );
        assert!(
            output
                .versions
                .contains(&VersionSpec::parse("3.24.1").unwrap())
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn sets_latest_alias_to_newest_revision() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("flutter-tizen-test").await;

        let output = plugin.load_versions(LoadVersionsInput::default()).await;
        let newest = output
            .versions
            .iter()
            .max()
            .map(|version| version.to_unresolved_spec());

        assert!(output.latest.is_some());
        assert_eq!(output.latest, newest);
        assert_eq!(output.aliases.get("latest"), output.latest.as_ref());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn rewrites_tags_and_bare_versions() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("flutter-tizen-test").await;

        let resolve = async |initial: &str| {
            plugin
                .resolve_version(ResolveVersionInput {
                    initial: UnresolvedVersionSpec::parse(initial).unwrap(),
                    ..Default::default()
                })
                .await
        };

        assert_eq!(
            resolve("3.47.1-tizen.1.1.1").await.candidate,
            UnresolvedVersionSpec::parse("3.47.1+tizen.1.1.1").ok()
        );
        assert_eq!(
            resolve("3.47.1").await.candidate,
            UnresolvedVersionSpec::parse("=3.47.1").ok()
        );
        assert_eq!(resolve("3.47").await, ResolveVersionOutput::default());
        assert_eq!(resolve("latest").await, ResolveVersionOutput::default());
    }
}
