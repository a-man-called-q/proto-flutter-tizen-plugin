use proto_pdk_test_utils::*;

mod flutter_tizen_tool {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn registers_metadata() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("flutter-tizen-test").await;

        let metadata = plugin
            .register_tool(RegisterToolInput {
                id: Id::raw("flutter-tizen"),
            })
            .await;

        assert_eq!(metadata.name, "flutter-tizen");
        assert_eq!(metadata.type_of, PluginType::CommandLine);
        assert_eq!(metadata.self_upgrade_commands, vec!["upgrade", "channel"]);
        assert_eq!(
            metadata.plugin_version.unwrap().to_string(),
            env!("CARGO_PKG_VERSION")
        );
    }
}
