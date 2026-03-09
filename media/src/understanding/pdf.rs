use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{
    AnalysisRequest, AnalysisResult, MediaSource, MediaType, UnderstandingClient, VisionProvider,
};
use crate::{MediaError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdfFallbackMode {
    TextFirst,
    ImagesFirst,
    TextOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfPageRange {
    pub from: u32,
    pub to: u32,
}

impl PdfPageRange {
    pub fn normalize(self) -> Self {
        if self.from <= self.to {
            self
        } else {
            Self {
                from: self.to,
                to: self.from,
            }
        }
    }
}

impl UnderstandingClient {
    pub async fn analyze_pdf_with_fallback(
        &self,
        request: AnalysisRequest,
        page_range: Option<PdfPageRange>,
        mode: PdfFallbackMode,
    ) -> Result<AnalysisResult> {
        let provider = self.select_provider(MediaType::Pdf, request.preferred_provider)?;
        match provider {
            VisionProvider::Anthropic | VisionProvider::Google => {
                let remote_result = match provider {
                    VisionProvider::Anthropic => self.anthropic_pdf(request.clone()).await,
                    VisionProvider::Google => self.google_pdf(request.clone()).await,
                    _ => unreachable!(),
                };
                if let Ok(result) = remote_result {
                    return Ok(result);
                }
            }
            _ => {}
        }

        let path = request.source.file_path().ok_or_else(|| {
            MediaError::InvalidInput("pdf fallback requires file source".to_string())
        })?;

        match mode {
            PdfFallbackMode::TextOnly => {
                let text = self.run_pdftotext(path.clone(), page_range).await?;
                Ok(AnalysisResult {
                    media_type: MediaType::Pdf,
                    provider: "local-pdftotext".to_string(),
                    text,
                    metadata: json!({"mode": "text_only"}),
                })
            }
            PdfFallbackMode::TextFirst => {
                let text = self.run_pdftotext(path.clone(), page_range).await?;
                if text.split_whitespace().count() >= 40 {
                    return Ok(AnalysisResult {
                        media_type: MediaType::Pdf,
                        provider: "local-pdftotext".to_string(),
                        text,
                        metadata: json!({"mode": "text_first"}),
                    });
                }
                self.pdf_via_images(path, request.prompt, page_range).await
            }
            PdfFallbackMode::ImagesFirst => {
                let image_attempt = self
                    .pdf_via_images(path.clone(), request.prompt.clone(), page_range)
                    .await;
                if image_attempt.is_ok() {
                    image_attempt
                } else {
                    let text = self.run_pdftotext(path, page_range).await?;
                    Ok(AnalysisResult {
                        media_type: MediaType::Pdf,
                        provider: "local-pdftotext".to_string(),
                        text,
                        metadata: json!({"mode": "images_first_fallback_text"}),
                    })
                }
            }
        }
    }

    async fn run_pdftotext(
        &self,
        path: PathBuf,
        page_range: Option<PdfPageRange>,
    ) -> Result<String> {
        let mut cmd = tokio::process::Command::new("pdftotext");
        cmd.arg("-layout");
        if let Some(range) = page_range.map(PdfPageRange::normalize) {
            cmd.arg("-f").arg(range.from.to_string());
            cmd.arg("-l").arg(range.to.to_string());
        }
        cmd.arg(path.as_os_str()).arg("-");

        let output = cmd
            .output()
            .await
            .map_err(|e| MediaError::Execution(format!("pdftotext spawn failed: {e}")))?;
        if !output.status.success() {
            return Err(MediaError::Execution(format!(
                "pdftotext failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn pdf_via_images(
        &self,
        path: PathBuf,
        prompt: String,
        page_range: Option<PdfPageRange>,
    ) -> Result<AnalysisResult> {
        let temp = tempfile::Builder::new()
            .prefix("media-pdf-pages-")
            .tempdir()
            .map_err(MediaError::Io)?;

        let mut cmd = tokio::process::Command::new("pdftoppm");
        cmd.arg("-jpeg");
        if let Some(range) = page_range.map(PdfPageRange::normalize) {
            cmd.arg("-f").arg(range.from.to_string());
            cmd.arg("-l").arg(range.to.to_string());
        }
        let out_prefix = temp.path().join("page");
        cmd.arg(path.as_os_str()).arg(out_prefix.as_os_str());

        let output = cmd
            .output()
            .await
            .map_err(|e| MediaError::Execution(format!("pdftoppm spawn failed: {e}")))?;
        if !output.status.success() {
            return Err(MediaError::Execution(format!(
                "pdftoppm failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let mut images = Vec::new();
        let mut read = tokio::fs::read_dir(temp.path()).await?;
        while let Some(entry) = read.next_entry().await? {
            if !entry.file_type().await?.is_file() {
                continue;
            }
            let path = entry.path();
            let is_jpeg = path
                .extension()
                .and_then(|x| x.to_str())
                .map(|x| x.eq_ignore_ascii_case("jpg") || x.eq_ignore_ascii_case("jpeg"))
                .unwrap_or(false);
            if is_jpeg {
                images.push(path);
            }
        }
        images.sort();

        if images.is_empty() {
            return Err(MediaError::Execution(
                "pdftoppm produced no page images".to_string(),
            ));
        }

        let mut summaries = Vec::new();
        for (idx, image) in images.iter().enumerate().take(12) {
            let analysis = self
                .analyze_image(
                    AnalysisRequest {
                        media_type: MediaType::Image,
                        source: MediaSource::File {
                            path: image.clone(),
                        },
                        prompt: format!("{}\n\nPDF page {}", prompt, idx + 1),
                        preferred_provider: None,
                        metadata: Value::Null,
                    },
                    self.select_provider(MediaType::Image, None)?,
                )
                .await?;
            summaries.push(format!("Page {}: {}", idx + 1, analysis.text));
        }

        Ok(AnalysisResult {
            media_type: MediaType::Pdf,
            provider: "local-pdftoppm+vision".to_string(),
            text: summaries.join("\n\n"),
            metadata: json!({
                "image_pages": images.len(),
                "mode": "vision_fallback",
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_normalization_swaps_when_needed() {
        let r = PdfPageRange { from: 5, to: 2 }.normalize();
        assert_eq!(r.from, 2);
        assert_eq!(r.to, 5);
    }
}
