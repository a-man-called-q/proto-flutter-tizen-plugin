use flutter_tizen_plugin::FlutterTizenToolConfig;
use proto_pdk_test_utils::*;
use std::path::Path;
use std::process::Command;

mod flutter_tizen_tool {
    use super::*;

    async fn install(sandbox: &ProtoWasmSandbox, skip_bootstrap: bool) -> std::path::PathBuf {
        let mut plugin = sandbox
            .create_plugin_with_config("flutter-tizen-test", |config| {
                config.tool_config(FlutterTizenToolConfig {
                    skip_bootstrap,
                    ..Default::default()
                });
            })
            .await;
        let mut spec = ToolSpec::parse("3.44.4-tizen.1.0.0").unwrap();

        flow::manage::Manager::new(&mut plugin.tool)
            .install(&mut spec, flow::install::InstallOptions::default())
            .await
            .unwrap();

        check_install_success!(plugin, spec);

        plugin.tool.get_product_dir(&spec)
    }

    fn git(dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();

        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn clones_the_release_tag() {
        let sandbox = create_empty_proto_sandbox();
        let dir = install(&sandbox, true).await;

        assert!(dir.ends_with("3.44.4+tizen.1.0.0"));
        assert!(dir.join("bin/flutter-tizen").exists());
        assert!(dir.join("bin/flutter-tizen.bat").exists());
        assert!(dir.join("bin/internal/flutter.version").exists());

        // The launcher stamps its snapshot with the checkout's revision
        assert_eq!(
            git(&dir, &["describe", "--tags", "--exact-match"]),
            "3.44.4-tizen.1.0.0"
        );

        // The Flutter SDK is left for the first run
        assert!(!dir.join("flutter").exists());
    }

    // Clones the Flutter SDK, which is multiple gigabytes
    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn bootstraps_the_flutter_sdk() {
        let sandbox = create_empty_proto_sandbox();
        let dir = install(&sandbox, false).await;

        assert!(dir.join("flutter/.git").exists());
        assert!(dir.join("bin/cache/flutter-tizen.snapshot").exists());

        let revision = std::fs::read_to_string(dir.join("bin/internal/flutter.version")).unwrap();

        assert_eq!(
            git(&dir.join("flutter"), &["rev-parse", "HEAD"]),
            revision.trim()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn errors_for_unknown_tag() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox.create_plugin("flutter-tizen-test").await;
        let install_dir = sandbox.path().join("install");

        std::fs::create_dir_all(&install_dir).unwrap();

        let output = plugin
            .native_install(NativeInstallInput {
                context: PluginContext {
                    version: VersionSpec::parse("99.99.99").unwrap(),
                    ..Default::default()
                },
                install_dir: plugin.tool.to_virtual_path(&install_dir),
                ..Default::default()
            })
            .await;

        assert!(!output.installed);
        assert!(output.error.unwrap().contains("99.99.99"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn errors_for_unsupported_host() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("flutter-tizen-test", |config| {
                config.host(HostOS::Linux, HostArch::Arm64);
            })
            .await;

        let result = plugin
            .tool
            .plugin
            .call_func_with::<_, _, NativeInstallOutput>(
                PluginFunction::NativeInstall,
                NativeInstallInput {
                    context: PluginContext {
                        version: VersionSpec::parse("3.44.4+tizen.1.0.0").unwrap(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn locates_unix_bin() {
        for (os, arch) in [
            (HostOS::Linux, HostArch::X64),
            (HostOS::MacOS, HostArch::Arm64),
        ] {
            let sandbox = create_empty_proto_sandbox();
            let plugin = sandbox
                .create_plugin_with_config("flutter-tizen-test", |config| {
                    config.host(os, arch);
                })
                .await;

            let output = plugin
                .locate_executables(LocateExecutablesInput::default())
                .await;
            let exe = output.exes.get("flutter-tizen").unwrap();

            assert!(exe.primary);
            assert_eq!(exe.exe_path, Some("bin/flutter-tizen".into()));
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn locates_windows_bin() {
        let sandbox = create_empty_proto_sandbox();
        let plugin = sandbox
            .create_plugin_with_config("flutter-tizen-test", |config| {
                config.host(HostOS::Windows, HostArch::X64);
            })
            .await;

        let output = plugin
            .locate_executables(LocateExecutablesInput::default())
            .await;

        assert_eq!(
            output.exes.get("flutter-tizen").unwrap().exe_path,
            Some("bin/flutter-tizen.bat".into())
        );
    }
}
