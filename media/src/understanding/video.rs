use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{
    AnalysisRequest, AnalysisResult, MediaSource, MediaType, UnderstandingClient, VisionProvider,
};
use crate::{MediaError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyFrame {
    pub index: usize,
    pub path: PathBuf,
    pub timestamp_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoPlan {
    pub frame_count: usize,
    pub fps: f32,
    pub max_seconds: Option<u64>,
}

impl Default for VideoPlan {
    fn default() -> Self {
        Self {
            frame_count: 8,
            fps: 1.0,
            max_seconds: Some(120),
        }
    }
}

impl UnderstandingClient {
    pub async fn extract_key_frames(&self, path: &Path, plan: VideoPlan) -> Result<Vec<KeyFrame>> {
        let dir = tempfile::Builder::new()
            .prefix("media-video-frames-")
            .tempdir()
            .map_err(MediaError::Io)?;
        let pattern = dir.path().join("frame-%03d.jpg");

        let mut cmd = tokio::process::Command::new("ffmpeg");
        cmd.arg("-hide_banner")
            .arg("-loglevel")
            .arg("error")
            .arg("-ss")
            .arg("0")
            .arg("-i")
            .arg(path)
            .arg("-vf")
            .arg(format!("fps={}", plan.fps.max(0.1)))
            .arg("-vframes")
            .arg(plan.frame_count.to_string())
            .arg(pattern.as_os_str());

        if let Some(max_seconds) = plan.max_seconds {
            cmd.arg("-t").arg(max_seconds.to_string());
        }

        let out = cmd
            .output()
            .await
            .map_err(|e| MediaError::Execution(format!("ffmpeg execute failed: {e}")))?;
        if !out.status.success() {
            return Err(MediaError::Execution(format!(
                "ffmpeg frame extraction failed: {}",
                String::from_utf8_lossy(&out.stderr)
            )));
        }

        let mut files = Vec::new();
        let mut read = tokio::fs::read_dir(dir.path()).await?;
        while let Some(entry) = read.next_entry().await? {
            if entry.file_type().await?.is_file() {
                files.push(entry.path());
            }
        }
        files.sort();

        let stash = std::env::temp_dir().join(format!(
            "magicmerlin-video-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));
        tokio::fs::create_dir_all(&stash).await?;

        let mut frames = Vec::new();
        for (idx, file) in files.into_iter().enumerate() {
            let target = stash.join(
                file.file_name()
                    .ok_or_else(|| MediaError::Execution("frame without file name".to_string()))?,
            );
            tokio::fs::copy(&file, &target).await?;
            frames.push(KeyFrame {
                index: idx,
                path: target,
                timestamp_secs: idx as f64 / f64::from(plan.fps.max(0.1)),
            });
        }

        Ok(frames)
    }

    pub async fn analyze_video_with_plan(
        &self,
        request: AnalysisRequest,
        provider: VisionProvider,
        plan: VideoPlan,
    ) -> Result<AnalysisResult> {
        let path = request.source.file_path().ok_or_else(|| {
            MediaError::InvalidInput("video analysis requires file source".to_string())
        })?;

        let frames = self
            .extract_key_frames(path.as_path(), plan.clone())
            .await?;
        if frames.is_empty() {
            return Err(MediaError::Execution(
                "no frames extracted from video".to_string(),
            ));
        }

        let mut parts = Vec::with_capacity(frames.len());
        for frame in &frames {
            let frame_request = AnalysisRequest {
                media_type: MediaType::Image,
                source: MediaSource::File {
                    path: frame.path.clone(),
                },
                prompt: format!(
                    "{}\n\nFrame {} @ {:.1}s",
                    request.prompt,
                    frame.index + 1,
                    frame.timestamp_secs
                ),
                preferred_provider: Some(provider),
                metadata: json!({
                    "frame_index": frame.index,
                    "timestamp_secs": frame.timestamp_secs,
                }),
            };
            let result = self.analyze_image(frame_request, provider).await?;
            parts.push(format!(
                "Frame {} ({:.1}s): {}",
                frame.index + 1,
                frame.timestamp_secs,
                result.text
            ));
        }

        Ok(AnalysisResult {
            media_type: MediaType::Video,
            provider: format!("{:?}", provider).to_lowercase(),
            text: parts.join("\n\n"),
            metadata: json!({
                "plan": plan,
                "frame_count": frames.len(),
                "frames": frames,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_video_plan_is_sane() {
        let plan = VideoPlan::default();
        assert!(plan.frame_count >= 1);
        assert!(plan.fps > 0.0);
    }
}
