use crate::config::FlutterTizenToolConfig;
use crate::version::{normalize_spec, version_to_tag, versions_from_tags};
use extism_pdk::*;
use proto_pdk::*;
use schematic::SchemaBuilder;
use std::collections::HashMap;

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
}

static NAME: &str = "flutter-tizen";

#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(RegisterToolOutput {
        name: NAME.into(),
        type_of: PluginType::CommandLine,
        minimum_proto_version: Some(Version::new(0, 60, 0)),
        plugin_version: Version::parse(env!("CARGO_PKG_VERSION")).ok(),
        // Both move the Git checkout to another tag
        self_upgrade_commands: vec!["upgrade".into(), "channel".into()],
        ..Default::default()
    }))
}

#[plugin_fn]
pub fn define_tool_config(_: ()) -> FnResult<Json<DefineToolConfigOutput>> {
    Ok(Json(DefineToolConfigOutput {
        schema: SchemaBuilder::build_root::<FlutterTizenToolConfig>(),
    }))
}

#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let config = get_tool_config::<FlutterTizenToolConfig>()?;
    let versions = versions_from_tags(load_git_tags(&config.repo_url)?);
    let mut output = LoadVersionsOutput::default();

    if let Some(latest) = versions.last() {
        let latest = UnresolvedVersionSpec::Version(latest.to_owned());

        output.aliases.insert("latest".into(), latest.clone());
        output.latest = Some(latest);
    }

    output.versions = versions.into_iter().map(VersionSpec::Version).collect();

    Ok(Json(output))
}

#[plugin_fn]
pub fn resolve_version(
    Json(input): Json<ResolveVersionInput>,
) -> FnResult<Json<ResolveVersionOutput>> {
    Ok(Json(ResolveVersionOutput {
        candidate: normalize_spec(&input.initial),
        ..Default::default()
    }))
}

// flutter-tizen has no pre-built archives, and its launcher reads the Git
// revision of its own checkout, so it must be cloned.
#[plugin_fn]
pub fn native_install(
    Json(input): Json<NativeInstallInput>,
) -> FnResult<Json<NativeInstallOutput>> {
    let env = get_host_environment()?;

    check_supported_os_and_arch(
        NAME,
        env,
        permutations! [
            HostOS::Linux => [HostArch::X64],
            HostOS::MacOS => [HostArch::X64, HostArch::Arm64],
            HostOS::Windows => [HostArch::X64],
        ],
    )?;

    let failed = |error: String| {
        Ok(Json(NativeInstallOutput {
            error: Some(error),
            ..Default::default()
        }))
    };

    let Some(version) = input.context.version.as_version() else {
        return failed(format!(
            "Only tagged releases of {NAME} can be installed, received <id>{}</id>.",
            input.context.version
        ));
    };

    if !command_exists(env, "git") {
        return failed(format!(
            "{NAME} is installed with Git, but <shell>git</shell> was not found on <env>PATH</env>."
        ));
    }

    let config = get_tool_config::<FlutterTizenToolConfig>()?;
    let tag = version_to_tag(version);

    let result = exec(ExecCommandInput {
        command: "git".into(),
        args: vec![
            "-c".into(),
            "advice.detachedHead=false".into(),
            "clone".into(),
            "--depth".into(),
            "1".into(),
            "--branch".into(),
            tag.clone(),
            config.repo_url.clone(),
            ".".into(),
        ],
        cwd: Some(input.install_dir.clone()),
        stream: true,
        ..Default::default()
    })?;

    if result.exit_code != 0 {
        return failed(format!(
            "Failed to clone tag <id>{tag}</id> from <url>{}</url>.",
            config.repo_url
        ));
    }

    // The first run clones the Flutter SDK and compiles the tool, so do it now
    // instead of from a shim, where parallel runs would clash.
    if !config.skip_bootstrap {
        let result = exec(ExecCommandInput {
            command: input
                .install_dir
                .join(
                    env.os
                        .for_native("bin/flutter-tizen", "bin/flutter-tizen.bat"),
                )
                .to_string(),
            args: vec!["--version".into()],
            cwd: Some(input.install_dir.clone()),
            stream: true,
            ..Default::default()
        })?;

        if result.exit_code != 0 {
            return failed(format!(
                "Failed to set up the Flutter SDK for {NAME} <id>{tag}</id>."
            ));
        }
    }

    Ok(Json(NativeInstallOutput {
        installed: true,
        ..Default::default()
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;

    Ok(Json(LocateExecutablesOutput {
        exes: HashMap::from_iter([(
            NAME.into(),
            ExecutableConfig::new_primary(
                env.os
                    .for_native("bin/flutter-tizen", "bin/flutter-tizen.bat"),
            ),
        )]),
        exes_dirs: vec!["bin".into()],
        ..Default::default()
    }))
}
