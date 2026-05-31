//! Tests for tokenomics::multimodal
//! Extracted from inline #[cfg(test)] module

use hudhudscript_tokenomics::multimodal::*;

#[test]
fn test_low_detail_fixed_tokens() {
    let opt = MultiModalOptimizer::new();
    assert_eq!(opt.image_token_count(4000, 3000, ImageDetail::Low), 85);
    assert_eq!(opt.image_token_count(100, 100, ImageDetail::Low), 85);
}

#[test]
fn test_high_detail_small_image() {
    let opt = MultiModalOptimizer::new();
    // 512x512: fits in 2048, short side 512 <= 768 so no scaling.
    // 1 tile x 1 tile = 170 + 85 = 255
    let tokens = opt.image_token_count(512, 512, ImageDetail::High);
    assert_eq!(tokens, 255);
}

#[test]
fn test_high_detail_large_image() {
    let opt = MultiModalOptimizer::new();
    // 4096x4096 -> scale_to_fit(2048) -> 2048x2048
    // scale_short_side(768) -> 768x768
    // tiles: ceil(768/512) = 2 x 2 = 4 tiles
    // 4 * 170 + 85 = 765
    let tokens = opt.image_token_count(4096, 4096, ImageDetail::High);
    assert_eq!(tokens, 765);
}

#[test]
fn test_image_savings_low_vs_high() {
    let opt = MultiModalOptimizer::new();
    let savings = opt.image_savings(2048, 2048, ImageDetail::High, ImageDetail::Low);
    assert!(savings.tokens_saved > 0);
    assert!(savings.savings_pct > 50.0);
}

#[test]
fn test_audio_comparison_recommends_cheaper() {
    let opt = MultiModalOptimizer::new();
    let cmp = opt.audio_cost_comparison(
        60.0,  // 1 minute
        128.0, // tokens/sec for raw audio
        2.5,   // words/sec
        1.3,   // tokens/word
        0.06,  // audio cost per 1k
        0.03,  // text cost per 1k
        0.006, // whisper cost per minute
    );
    // Raw: 7680 tokens * 0.06/1k = 0.4608
    // Transcribe: 195 tokens * 0.03/1k + 0.006 ~ 0.01185
    assert_eq!(cmp.recommended, AudioStrategy::TranscribeFirst);
    assert!(cmp.total_transcribe_cost_usd < cmp.raw_cost_usd);
}

#[test]
fn test_video_frame_tokens() {
    let opt = MultiModalOptimizer::new();
    // 10 seconds at 1 fps = 10 frames, low detail = 85 each
    let tokens = opt.video_frame_tokens(10.0, 1.0, 1920, 1080, ImageDetail::Low);
    assert_eq!(tokens, 850);
}

#[test]
fn test_scale_to_fit_no_change() {
    assert_eq!(scale_to_fit(800, 600, 2048), (800, 600));
}

#[test]
fn test_scale_to_fit_landscape() {
    let (w, h) = scale_to_fit(4000, 2000, 2048);
    assert!(w <= 2048);
    assert!(h <= 2048);
    // aspect ratio preserved approximately
    let ratio = w as f64 / h as f64;
    assert!((ratio - 2.0).abs() < 0.1);
}

#[test]
fn test_scale_short_side_already_small() {
    assert_eq!(scale_short_side(500, 300, 768), (500, 300));
}

#[test]
fn test_scale_short_side_large() {
    let (w, h) = scale_short_side(2048, 1024, 768);
    assert_eq!(h.min(w), 768);
}

#[test]
fn test_optimizer_default() {
    let opt = MultiModalOptimizer::default();
    assert_eq!(opt.image_detail, ImageDetail::Auto);
    assert_eq!(opt.max_image_dimension, 2048);
}

#[test]
fn test_medium_detail_token_count() {
    // Lines 76-79: Medium detail calculation
    let opt = MultiModalOptimizer::new();
    // 1024x1024: fits in 2048, no short-side scaling for Medium
    // tiles_x = ceil(1024/512) = 2, tiles_y = ceil(1024/512) = 2
    // 2 * 2 * 170 + 85 = 765
    let tokens = opt.image_token_count(1024, 1024, ImageDetail::Medium);
    assert_eq!(tokens, 765);
}

#[test]
fn test_medium_detail_large_image() {
    // Lines 76-79: Medium detail with image exceeding 2048
    let opt = MultiModalOptimizer::new();
    // 4096x2048 -> scale_to_fit(2048) -> 2048x1024
    // tiles_x = ceil(2048/512) = 4, tiles_y = ceil(1024/512) = 2
    // 4 * 2 * 170 + 85 = 1445
    let tokens = opt.image_token_count(4096, 2048, ImageDetail::Medium);
    assert_eq!(tokens, 1445);
}

#[test]
fn test_image_savings_zero_original_tokens() {
    // Line 115: savings_pct when original_tokens == 0
    // A zero-dimension image is tricky; we use Low detail (85 tokens) for both.
    // Actually, original_tokens can never be 0 with real detail levels.
    // But we can get savings_pct = 0.0 when tokens_saved = 0.
    let opt = MultiModalOptimizer::new();
    let savings = opt.image_savings(100, 100, ImageDetail::Low, ImageDetail::Low);
    // Both are 85 tokens, no savings
    assert_eq!(savings.original_tokens, 85);
    assert_eq!(savings.optimized_tokens, 85);
    assert_eq!(savings.tokens_saved, 0);
    assert_eq!(savings.savings_pct, 0.0);
}

#[test]
fn test_audio_comparison_recommends_raw() {
    // Line 157: AudioStrategy::Raw when transcribe is more expensive
    let opt = MultiModalOptimizer::new();
    let cmp = opt.audio_cost_comparison(
        10.0,  // 10 seconds
        1.0,   // very low audio tokens/sec
        2.5,   // words/sec
        1.3,   // tokens/word
        0.001, // very cheap audio cost per 1k
        0.03,  // text cost per 1k
        1.0,   // very expensive whisper cost per minute
    );
    // Raw: 10 tokens * 0.001/1k = 0.00001
    // Transcribe: 33 tokens * 0.03/1k + (10/60)*1.0 ≈ 0.00099 + 0.1667 ≈ 0.1677
    assert_eq!(cmp.recommended, AudioStrategy::Raw);
    assert!(cmp.total_transcribe_cost_usd > cmp.raw_cost_usd);
}

#[test]
fn test_scale_to_fit_portrait() {
    // Line 207: scale_to_fit when height > width
    let (w, h) = scale_to_fit(2000, 4000, 2048);
    assert!(h <= 2048);
    assert!(w <= 2048);
    // height was larger, so it gets scaled to 2048
    assert_eq!(h, 2048);
    assert_eq!(w, 1024);
}

#[test]
fn test_scale_short_side_short_equals_target() {
    // Line 217-218: short <= target returns unchanged
    // short side == target exactly
    let (w, h) = scale_short_side(1024, 768, 768);
    assert_eq!(w, 1024);
    assert_eq!(h, 768);
}

// NOTE: multimodal.rs line 115 (`original_tokens == 0` guard in image_savings)
// is dead code. image_token_count() always returns at least 85 (Low detail),
// so original_tokens is never 0.
