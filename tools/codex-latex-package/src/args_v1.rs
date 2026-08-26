//! Parses the bounded packaging command line.

use std::ffi::OsString;
use std::path::PathBuf;

use crate::bundle_v1::StageOptionsV1;
use crate::error_v1::PackageErrorV1;
use crate::install_v1::InstallOptionsV1;
use crate::install_v1::UninstallOptionsV1;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Selects one versioned packaging operation.
pub enum CommandV1 {
    /// Stages a new developer bundle at an unused path.
    Stage(StageOptionsV1),
    /// Installs a verified bundle under an explicit prefix.
    Install(InstallOptionsV1),
    /// Removes the active manifest owned bundle from a prefix.
    Uninstall(UninstallOptionsV1),
}

/// Parses packaging arguments without consulting process state.
pub fn parse_command_v1(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<CommandV1, PackageErrorV1> {
    let mut arguments = arguments.into_iter();
    let command = arguments
        .next()
        .ok_or_else(|| usage_error("missing command"))?;
    match command.to_str() {
        Some("stage") => parse_stage_v1(arguments),
        Some("install") => parse_install_v1(arguments),
        Some("uninstall") => parse_uninstall_v1(arguments),
        _ => Err(usage_error("unknown or non Unicode command")),
    }
}

fn parse_stage_v1(
    mut arguments: impl Iterator<Item = OsString>,
) -> Result<CommandV1, PackageErrorV1> {
    let mut options = ParsedStageOptionsV1::default();
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| usage_error("every option requires a value"))?;
        match flag.to_str() {
            Some("--codex-binary") => set_once(&mut options.codex_binary, value, &flag)?,
            Some("--renderer-binary") => {
                set_once(&mut options.renderer_binary, value, &flag)?;
            }
            Some("--worker-dist") => set_once(&mut options.worker_dist, value, &flag)?,
            Some("--mathjax-module") => set_once(&mut options.mathjax_module, value, &flag)?,
            Some("--output") => set_once(&mut options.output, value, &flag)?,
            Some("--version") => set_once(&mut options.version, value, &flag)?,
            Some("--target") => set_once(&mut options.target, value, &flag)?,
            _ => return Err(usage_error("unknown or non Unicode option")),
        }
    }

    Ok(CommandV1::Stage(options.finish()?))
}

fn parse_install_v1(
    mut arguments: impl Iterator<Item = OsString>,
) -> Result<CommandV1, PackageErrorV1> {
    let mut bundle = None;
    let mut prefix = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| usage_error("every option requires a value"))?;
        match flag.to_str() {
            Some("--bundle") => set_once(&mut bundle, value, &flag)?,
            Some("--prefix") => set_once(&mut prefix, value, &flag)?,
            _ => return Err(usage_error("unknown or non Unicode option")),
        }
    }
    Ok(CommandV1::Install(InstallOptionsV1 {
        bundle: required_path(bundle, "--bundle")?,
        prefix: required_path(prefix, "--prefix")?,
    }))
}

fn parse_uninstall_v1(
    mut arguments: impl Iterator<Item = OsString>,
) -> Result<CommandV1, PackageErrorV1> {
    let mut prefix = None;
    while let Some(flag) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| usage_error("every option requires a value"))?;
        match flag.to_str() {
            Some("--prefix") => set_once(&mut prefix, value, &flag)?,
            _ => return Err(usage_error("unknown or non Unicode option")),
        }
    }
    Ok(CommandV1::Uninstall(UninstallOptionsV1 {
        prefix: required_path(prefix, "--prefix")?,
    }))
}

#[derive(Default)]
struct ParsedStageOptionsV1 {
    codex_binary: Option<OsString>,
    renderer_binary: Option<OsString>,
    worker_dist: Option<OsString>,
    mathjax_module: Option<OsString>,
    output: Option<OsString>,
    version: Option<OsString>,
    target: Option<OsString>,
}

impl ParsedStageOptionsV1 {
    fn finish(self) -> Result<StageOptionsV1, PackageErrorV1> {
        Ok(StageOptionsV1 {
            codex_binary: required_path(self.codex_binary, "--codex-binary")?,
            renderer_binary: required_path(self.renderer_binary, "--renderer-binary")?,
            worker_dist: required_path(self.worker_dist, "--worker-dist")?,
            mathjax_module: required_path(self.mathjax_module, "--mathjax-module")?,
            output: required_path(self.output, "--output")?,
            version: required_text(self.version, "--version")?,
            target: required_text(self.target, "--target")?,
        })
    }
}

fn set_once(
    destination: &mut Option<OsString>,
    value: OsString,
    flag: &OsString,
) -> Result<(), PackageErrorV1> {
    if destination.replace(value).is_some() {
        return Err(usage_error(&format!(
            "duplicate option {}",
            flag.to_string_lossy()
        )));
    }
    Ok(())
}

fn required_path(value: Option<OsString>, name: &str) -> Result<PathBuf, PackageErrorV1> {
    value
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .ok_or_else(|| usage_error(&format!("missing {name}")))
}

fn required_text(value: Option<OsString>, name: &str) -> Result<String, PackageErrorV1> {
    value
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| usage_error(&format!("missing or non Unicode {name}")))
}

fn usage_error(reason: &str) -> PackageErrorV1 {
    PackageErrorV1::new(format!(
        "{reason}. Usage: codex-latex-package stage OPTIONS | install --bundle PATH --prefix PATH | uninstall --prefix PATH"
    ))
}

#[cfg(test)]
#[path = "args_v1_tests.rs"]
mod tests;
