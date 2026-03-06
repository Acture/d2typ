mod latex;
mod request;
mod typst;

pub use request::{ArtifactKind, BackendKind, RenderRequest, RenderedArtifact};

use crate::core::Document;
use crate::error::{DocpackError, DocpackResult};

pub trait Backend {
    fn kind(&self) -> BackendKind;
    fn render(&self, doc: &Document, req: &RenderRequest) -> DocpackResult<RenderedArtifact>;
}

pub fn render_document(doc: &Document, req: &RenderRequest) -> DocpackResult<RenderedArtifact> {
    validate_request(doc, req)?;
    match req.backend {
        BackendKind::Typst => typst::TypstBackend.render(doc, req),
        BackendKind::Latex => latex::LatexBackend.render(doc, req),
    }
}

pub fn validate_request(doc: &Document, req: &RenderRequest) -> DocpackResult<()> {
    if req.artifact == ArtifactKind::TableFragment && !doc.is_tabular() {
        return Err(DocpackError::Render {
            backend: req.backend,
            artifact: req.artifact,
            detail: "artifact table-fragment requires tabular source metadata".to_string(),
        });
    }
    if !style_supported(req.backend, req.artifact, &req.style) {
        return Err(DocpackError::Render {
            backend: req.backend,
            artifact: req.artifact,
            detail: format!(
                "style '{}' is not supported for this backend/artifact",
                req.style
            ),
        });
    }
    Ok(())
}

pub fn style_supported(backend: BackendKind, artifact: ArtifactKind, style: &str) -> bool {
    matches!(
        (backend, artifact, style),
        (
            BackendKind::Typst,
            ArtifactKind::DataModule,
            "typst-official"
        ) | (
            BackendKind::Typst,
            ArtifactKind::TableFragment,
            "typst-table"
        ) | (BackendKind::Latex, ArtifactKind::DataModule, "latex-expl3")
            | (
                BackendKind::Latex,
                ArtifactKind::DataModule,
                "latex-classic-macro"
            )
            | (
                BackendKind::Latex,
                ArtifactKind::TableFragment,
                "latex-booktabs-longtable"
            )
            | (
                BackendKind::Latex,
                ArtifactKind::TableFragment,
                "latex-plain-tabular"
            )
    )
}

pub fn style_implied_backend(style: &str) -> Option<BackendKind> {
    match style {
        "typst-official" | "typst-table" => Some(BackendKind::Typst),
        "latex-expl3"
        | "latex-classic-macro"
        | "latex-booktabs-longtable"
        | "latex-plain-tabular" => Some(BackendKind::Latex),
        _ => None,
    }
}

pub fn style_implied_artifact(style: &str) -> Option<ArtifactKind> {
    match style {
        "typst-table" | "latex-booktabs-longtable" | "latex-plain-tabular" => {
            Some(ArtifactKind::TableFragment)
        }
        "typst-official" | "latex-expl3" | "latex-classic-macro" => Some(ArtifactKind::DataModule),
        _ => None,
    }
}

pub fn default_style(backend: BackendKind, artifact: ArtifactKind) -> &'static str {
    match (backend, artifact) {
        (BackendKind::Typst, ArtifactKind::DataModule) => "typst-official",
        (BackendKind::Typst, ArtifactKind::TableFragment) => "typst-table",
        (BackendKind::Latex, ArtifactKind::DataModule) => "latex-expl3",
        (BackendKind::Latex, ArtifactKind::TableFragment) => "latex-booktabs-longtable",
    }
}
