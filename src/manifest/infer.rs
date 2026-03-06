use std::path::{Path, PathBuf};

use crate::backend::{
    ArtifactKind, BackendKind, RenderRequest, default_style, style_implied_artifact,
    style_implied_backend, style_supported,
};
use crate::core::Document;
use crate::error::{DocpackError, DocpackResult};
use crate::manifest::{LoadedManifest, OutputEntry, SourceEntry};

#[derive(Debug, Clone)]
pub struct ResolvedOutput {
    pub output_id: String,
    pub source_id: String,
    pub output_path: PathBuf,
    pub request: RenderRequest,
}

#[derive(Debug, Clone, Copy)]
pub struct SourceRequestOptions<'a> {
    pub input_path: Option<&'a Path>,
    pub output_path: Option<&'a Path>,
    pub backend: Option<BackendKind>,
    pub artifact: Option<ArtifactKind>,
    pub style: Option<&'a str>,
    pub root_name: Option<&'a str>,
    pub require_explicit_backend_without_output: bool,
}

pub fn resolve_manifest_output(
    loaded: &LoadedManifest,
    source: &SourceEntry,
    output: &OutputEntry,
    doc: &Document,
) -> DocpackResult<ResolvedOutput> {
    let output_path = resolve_output_path(loaded, &output.path);
    let request = resolve_request(
        doc,
        ResolveRequestContext {
            output_path: Some(&output_path),
            backend: output.backend,
            artifact: output.artifact,
            style: output.style.as_deref(),
            root_name: output.root_name.as_deref(),
            source_id: Some(&source.id),
            input_path: None,
            require_explicit_backend_without_output: false,
        },
    )?;
    Ok(ResolvedOutput {
        output_id: output.id.clone(),
        source_id: source.id.clone(),
        output_path,
        request,
    })
}

pub fn resolve_source_request(
    doc: &Document,
    options: SourceRequestOptions<'_>,
) -> DocpackResult<RenderRequest> {
    resolve_request(
        doc,
        ResolveRequestContext {
            output_path: options.output_path,
            backend: options.backend,
            artifact: options.artifact,
            style: options.style,
            root_name: options.root_name,
            source_id: None,
            input_path: options.input_path,
            require_explicit_backend_without_output: options
                .require_explicit_backend_without_output,
        },
    )
}

pub fn sanitize_root_name(value: &str) -> String {
    let mut sanitized = String::new();
    let mut previous_underscore = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        let next = if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' {
            ch
        } else {
            '_'
        };
        if next == '_' && previous_underscore {
            continue;
        }
        previous_underscore = next == '_';
        sanitized.push(next);
    }
    let mut sanitized = sanitized.trim_matches('_').to_string();
    if sanitized
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_digit())
    {
        sanitized = format!("data_{sanitized}");
    }
    if sanitized.is_empty() {
        "data".to_string()
    } else {
        sanitized
    }
}

#[derive(Debug, Clone, Copy)]
struct ResolveRequestContext<'a> {
    output_path: Option<&'a Path>,
    backend: Option<BackendKind>,
    artifact: Option<ArtifactKind>,
    style: Option<&'a str>,
    root_name: Option<&'a str>,
    source_id: Option<&'a str>,
    input_path: Option<&'a Path>,
    require_explicit_backend_without_output: bool,
}

fn resolve_request(
    doc: &Document,
    context: ResolveRequestContext<'_>,
) -> DocpackResult<RenderRequest> {
    if context.output_path.is_none()
        && context.require_explicit_backend_without_output
        && context.backend.is_none()
    {
        return Err(DocpackError::Inference {
            detail: "stdout output requires explicit --backend".to_string(),
        });
    }

    let backend = infer_backend(context.output_path, context.backend, context.style)?;
    let artifact = infer_artifact(context.artifact, context.style);
    if artifact == ArtifactKind::TableFragment && !doc.is_tabular() {
        return Err(DocpackError::Inference {
            detail: "artifact table-fragment requires tabular source metadata".to_string(),
        });
    }
    let style = infer_style(backend, artifact, context.style);
    if !style_supported(backend, artifact, &style) {
        return Err(DocpackError::Inference {
            detail: format!(
                "style '{}' is not valid for {} {}",
                style, backend, artifact
            ),
        });
    }
    let root_name = infer_root_name(context.root_name, context.source_id, context.input_path);
    Ok(RenderRequest {
        backend,
        artifact,
        style,
        root_name,
    })
}

fn resolve_output_path(loaded: &LoadedManifest, path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    if let Some(output_dir) = loaded
        .manifest
        .project
        .as_ref()
        .and_then(|project| project.output_dir.as_ref())
    {
        loaded.dir.join(output_dir).join(path)
    } else {
        loaded.dir.join(path)
    }
}

fn infer_backend(
    output_path: Option<&Path>,
    backend: Option<BackendKind>,
    style: Option<&str>,
) -> DocpackResult<BackendKind> {
    if let Some(backend) = backend {
        return Ok(backend);
    }
    if let Some(path) = output_path {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("typ") => return Ok(BackendKind::Typst),
            Some("tex") => return Ok(BackendKind::Latex),
            _ => {}
        }
    }
    if let Some(style) = style.and_then(style_implied_backend) {
        return Ok(style);
    }
    Err(DocpackError::Inference {
        detail: "backend could not be inferred; provide --backend or an output path with .typ/.tex"
            .to_string(),
    })
}

fn infer_artifact(artifact: Option<ArtifactKind>, style: Option<&str>) -> ArtifactKind {
    artifact
        .or_else(|| style.and_then(style_implied_artifact))
        .unwrap_or(ArtifactKind::DataModule)
}

fn infer_style(backend: BackendKind, artifact: ArtifactKind, style: Option<&str>) -> String {
    style
        .map(str::to_string)
        .unwrap_or_else(|| default_style(backend, artifact).to_string())
}

fn infer_root_name(
    explicit: Option<&str>,
    source_id: Option<&str>,
    input_path: Option<&Path>,
) -> String {
    let raw = explicit
        .or(source_id)
        .or_else(|| input_path.and_then(|path| path.file_stem().and_then(|stem| stem.to_str())))
        .unwrap_or("data");
    sanitize_root_name(raw)
}

#[cfg(test)]
mod tests {
    use super::sanitize_root_name;

    #[test]
    fn sanitizes_root_names() {
        assert_eq!(sanitize_root_name("Quarterly Report"), "quarterly_report");
        assert_eq!(sanitize_root_name("2026-data"), "data_2026_data");
        assert_eq!(sanitize_root_name("---"), "data");
    }
}
