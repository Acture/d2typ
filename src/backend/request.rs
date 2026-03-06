use std::fmt::{Display, Formatter};

use clap::ValueEnum;
use serde::Deserialize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    Typst,
    Latex,
}

impl Display for BackendKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Typst => "typst",
            Self::Latex => "latex",
        };
        write!(f, "{value}")
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    DataModule,
    TableFragment,
}

impl Display for ArtifactKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::DataModule => "data-module",
            Self::TableFragment => "table-fragment",
        };
        write!(f, "{value}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderRequest {
    pub backend: BackendKind,
    pub artifact: ArtifactKind,
    pub style: String,
    pub root_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedArtifact {
    pub body: String,
}
