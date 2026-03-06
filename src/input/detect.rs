use std::path::Path;

use crate::core::{Origin, SourceFormat};
use crate::error::{DocpackError, DocpackResult};

pub fn detect_format(path: Option<&Path>) -> DocpackResult<SourceFormat> {
    match path {
        Some(path) => {
            SourceFormat::from_extension(path).ok_or_else(|| DocpackError::DetectFormat {
                origin: Origin::File(path.to_path_buf()),
                detail: "unknown or unsupported file extension".to_string(),
            })
        }
        None => Err(DocpackError::DetectFormat {
            origin: Origin::Stdin,
            detail: "stdin input requires an explicit --format".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::detect_format;
    use crate::core::SourceFormat;

    #[test]
    fn detects_known_extensions() {
        assert_eq!(
            detect_format(Some(Path::new("data.csv"))).unwrap(),
            SourceFormat::Csv
        );
        assert_eq!(
            detect_format(Some(Path::new("data.json"))).unwrap(),
            SourceFormat::Json
        );
        assert_eq!(
            detect_format(Some(Path::new("data.yml"))).unwrap(),
            SourceFormat::Yaml
        );
        assert_eq!(
            detect_format(Some(Path::new("data.toml"))).unwrap(),
            SourceFormat::Toml
        );
        assert_eq!(
            detect_format(Some(Path::new("data.xlsx"))).unwrap(),
            SourceFormat::Xlsx
        );
    }
}
