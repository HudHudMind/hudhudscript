//! Multi-modal token economics
//!
//! Calculates token costs for images, audio, and video inputs and provides
//! optimization strategies to reduce multi-modal API spend.

use serde::{Deserialize, Serialize};

/// Image detail level for vision models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageDetail {
    Low,
    Medium,
    High,
    Auto,
}

/// Strategy for handling audio inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioStrategy {
    /// Send raw audio directly to the model.
    Raw,
    /// Transcribe first and send text (cheaper but loses tone/nuance).
    TranscribeFirst,
}

/// Savings from optimizing image inputs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalSavings {
    pub original_tokens: u64,
    pub optimized_tokens: u64,
    pub tokens_saved: u64,
    pub savings_pct: f64,
}

/// Cost comparison between raw audio and transcribe-first strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCostComparison {
    pub raw_tokens: u64,
    pub raw_cost_usd: f64,
    pub transcribe_tokens: u64,
    pub transcribe_cost_usd: f64,
    pub whisper_cost_usd: f64,
    pub total_transcribe_cost_usd: f64,
    pub recommended: AudioStrategy,
}

/// Multi-modal optimizer for images, audio, and video.
#[derive(Debug, Clone)]
pub struct MultiModalOptimizer {
    pub image_detail: ImageDetail,
    pub max_image_dimension: u32,
    pub audio_strategy: AudioStrategy,
}

impl MultiModalOptimizer {
    pub fn new() -> Self {
        Self {
            image_detail: ImageDetail::Auto,
            max_image_dimension: 2048,
            audio_strategy: AudioStrategy::Raw,
        }
    }

    /// Estimate the number of tokens consumed by an image.
    ///
    /// Uses OpenAI's formula:
    /// - Low detail: fixed 85 tokens.
    /// - High detail: scale so short side is 768, then tile into 512x512 tiles,
    ///   each tile costs 170 tokens, plus 85 base tokens.
    pub fn image_token_count(&self, width: u32, height: u32, detail: ImageDetail) -> u64 {
        match detail {
            ImageDetail::Low => 85,
            ImageDetail::Medium => {
                // Medium: use high-detail formula but cap at 2048 with no
                // short-side scaling — a pragmatic middle ground.
                let (w, h) = scale_to_fit(width, height, 2048);
                let tiles_x = (w as f64 / 512.0).ceil() as u64;
                let tiles_y = (h as f64 / 512.0).ceil() as u64;
                tiles_x * tiles_y * 170 + 85
            }
            ImageDetail::High | ImageDetail::Auto => {
                // Step 1: scale to fit within 2048x2048
                let (w, h) = scale_to_fit(width, height, 2048);
                // Step 2: scale so shortest side is 768
                let (w, h) = scale_short_side(w, h, 768);
                // Step 3: count 512x512 tiles
                let tiles_x = (w as f64 / 512.0).ceil() as u64;
                let tiles_y = (h as f64 / 512.0).ceil() as u64;
                tiles_x * tiles_y * 170 + 85
            }
        }
    }

    /// Compute optimized dimensions that stay within `max_image_dimension`.
    pub fn optimize_image_dimensions(&self, width: u32, height: u32) -> (u32, u32) {
        scale_to_fit(width, height, self.max_image_dimension)
    }

    /// Estimate token savings from downgrading detail or resizing.
    pub fn image_savings(
        &self,
        width: u32,
        height: u32,
        original_detail: ImageDetail,
        optimized_detail: ImageDetail,
    ) -> MultiModalSavings {
        let original_tokens = self.image_token_count(width, height, original_detail);
        let (opt_w, opt_h) = self.optimize_image_dimensions(width, height);
        let optimized_tokens = self.image_token_count(opt_w, opt_h, optimized_detail);

        let tokens_saved = original_tokens.saturating_sub(optimized_tokens);
        let savings_pct = if original_tokens > 0 {
            (tokens_saved as f64 / original_tokens as f64) * 100.0
        } else {
            0.0
        };

        MultiModalSavings {
            original_tokens,
            optimized_tokens,
            tokens_saved,
            savings_pct,
        }
    }

    /// Compare costs of raw audio vs. transcribe-first.
    ///
    /// `duration_seconds` — length of the audio clip.
    /// `audio_tokens_per_second` — tokens consumed per second of raw audio.
    /// `words_per_second` — average speaking rate for transcript estimation.
    /// `tokens_per_word` — tokens per word in the transcript.
    /// `audio_cost_per_1k` — cost per 1k tokens for the audio model.
    /// `text_cost_per_1k` — cost per 1k tokens for the text model.
    /// `whisper_cost_per_minute` — Whisper API cost per minute.
    pub fn audio_cost_comparison(
        &self,
        duration_seconds: f64,
        audio_tokens_per_second: f64,
        words_per_second: f64,
        tokens_per_word: f64,
        audio_cost_per_1k: f64,
        text_cost_per_1k: f64,
        whisper_cost_per_minute: f64,
    ) -> AudioCostComparison {
        let raw_tokens = (duration_seconds * audio_tokens_per_second).ceil() as u64;
        let raw_cost = (raw_tokens as f64 / 1000.0) * audio_cost_per_1k;

        let word_count = duration_seconds * words_per_second;
        let transcribe_tokens = (word_count * tokens_per_word).ceil() as u64;
        let transcribe_cost = (transcribe_tokens as f64 / 1000.0) * text_cost_per_1k;
        let whisper_cost = (duration_seconds / 60.0) * whisper_cost_per_minute;
        let total_transcribe_cost = transcribe_cost + whisper_cost;

        let recommended = if total_transcribe_cost < raw_cost {
            AudioStrategy::TranscribeFirst
        } else {
            AudioStrategy::Raw
        };

        AudioCostComparison {
            raw_tokens,
            raw_cost_usd: raw_cost,
            transcribe_tokens,
            transcribe_cost_usd: transcribe_cost,
            whisper_cost_usd: whisper_cost,
            total_transcribe_cost_usd: total_transcribe_cost,
            recommended,
        }
    }

    /// Estimate the total tokens for a video as sampled frames.
    ///
    /// `duration_seconds` — video length.
    /// `fps_sample_rate` — frames sampled per second.
    /// `frame_width`, `frame_height` — resolution of each frame.
    /// `detail` — image detail level per frame.
    pub fn video_frame_tokens(
        &self,
        duration_seconds: f64,
        fps_sample_rate: f64,
        frame_width: u32,
        frame_height: u32,
        detail: ImageDetail,
    ) -> u64 {
        let frame_count = (duration_seconds * fps_sample_rate).ceil() as u64;
        let tokens_per_frame = self.image_token_count(frame_width, frame_height, detail);
        frame_count * tokens_per_frame
    }
}

impl Default for MultiModalOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Dimension helpers ──────────────────────────────────────────────────

/// Scale dimensions proportionally so neither exceeds `max_dim`.
pub fn scale_to_fit(width: u32, height: u32, max_dim: u32) -> (u32, u32) {
    if width <= max_dim && height <= max_dim {
        return (width, height);
    }
    let ratio = if width >= height {
        max_dim as f64 / width as f64
    } else {
        max_dim as f64 / height as f64
    };
    let new_w = (width as f64 * ratio).round() as u32;
    let new_h = (height as f64 * ratio).round() as u32;
    (new_w.max(1), new_h.max(1))
}

/// Scale so the shortest side equals `target`, maintaining aspect ratio.
pub fn scale_short_side(width: u32, height: u32, target: u32) -> (u32, u32) {
    let short = width.min(height);
    if short <= target {
        return (width, height);
    }
    let ratio = target as f64 / short as f64;
    let new_w = (width as f64 * ratio).round() as u32;
    let new_h = (height as f64 * ratio).round() as u32;
    (new_w.max(1), new_h.max(1))
}
