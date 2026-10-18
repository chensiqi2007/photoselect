use anyhow::{anyhow, bail, Context, Result};
use base64::Engine;
use image::{imageops::FilterType, ImageFormat};
use keyring::Entry;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Cursor;
use std::path::Path;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::decoder::ImageDecoder;
use crate::models::PhotoGroup;

const KEYRING_SERVICE: &str = "com.photoselect.desktop.ai";
const MAX_BATCH_SIZE: usize = 8;
const MAX_IMAGE_EDGE: u32 = 768;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiSelectionSettings {
    pub enabled: bool,
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiPhotoResult {
    pub id: String,
    pub score: u8,
    pub recommendation: String,
    pub reasons: Vec<String>,
    pub suggested_rating: u8,
    pub suggested_flag: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiBatchProgress {
    pub completed: usize,
    pub total: usize,
    pub batch: usize,
    pub batches: usize,
    pub results: Vec<AiPhotoResult>,
    pub error: Option<String>,
    pub finished: bool,
}

fn key_entry() -> Result<Entry> {
    Entry::new(KEYRING_SERVICE, "provider-api-key").context("Unable to access OS credential store")
}

fn api_key() -> Result<String> {
    key_entry()?.get_password().context("No API key is saved in the OS credential store")
}

pub fn load_settings(db: &crate::db::Database) -> Result<AiSelectionSettings> {
    let enabled = db.get_app_setting("ai_selection_enabled")?.as_deref() == Some("true");
    let base_url = db
        .get_app_setting("ai_selection_base_url")?
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
    let model = db
        .get_app_setting("ai_selection_model")?
        .unwrap_or_else(|| "gpt-4o-mini".to_string());
    let has_api_key = key_entry()
        .and_then(|entry| entry.get_password().map_err(Into::into))
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    Ok(AiSelectionSettings { enabled, base_url, model, has_api_key })
}

pub fn save_settings(
    db: &crate::db::Database,
    enabled: bool,
    base_url: &str,
    model: &str,
    new_api_key: Option<&str>,
    clear_api_key: bool,
) -> Result<AiSelectionSettings> {
    let base_url = base_url.trim().trim_end_matches('/');
    let model = model.trim();
    if !base_url.is_empty() {
        let parsed = reqwest::Url::parse(base_url).context("Invalid AI Base URL")?;
        if !matches!(parsed.scheme(), "https" | "http") || parsed.host_str().is_none() {
            bail!("Base URL must be an absolute HTTP or HTTPS URL");
        }
    }
    if !model.is_empty() && model.len() > 160 {
        bail!("Model identifier is too long");
    }

    db.set_app_settings(&[
        ("ai_selection_enabled", if enabled { "true" } else { "false" }),
        ("ai_selection_base_url", base_url),
        ("ai_selection_model", model),
    ])?;

    if new_api_key.map(str::trim).filter(|key| !key.is_empty()).is_some() || clear_api_key {
        let entry = key_entry()?;
        if let Some(key) = new_api_key.map(str::trim).filter(|key| !key.is_empty()) {
            entry.set_password(key).context("Unable to save API key in OS credential store")?;
        } else if clear_api_key {
            let _ = entry.delete_credential();
        }
    }
    load_settings(db)
}

pub async fn test_connection(db: &crate::db::Database) -> Result<String> {
    let settings = load_settings(db)?;
    if settings.base_url.is_empty() || settings.model.is_empty() {
        bail!("Set the Base URL and model first");
    }
    let key = api_key()?;
    let client = http_client()?;
    let url = format!("{}/models", settings.base_url);
    let response = client.get(url).bearer_auth(key).send().await.context("AI provider connection failed")?;
    if !response.status().is_success() {
        let status = response.status();
        bail!("Provider returned HTTP {status}");
    }
    Ok(format!("Connected to {}", settings.base_url))
}

pub async fn rank_photos(
    db: &crate::db::Database,
    app: &AppHandle,
    photos: Vec<PhotoGroup>,
) -> Result<()> {
    if photos.is_empty() {
        bail!("Select at least one photo to analyze");
    }
    if photos.len() > 500 {
        bail!("Analyze at most 500 photos per run");
    }
    let settings = load_settings(db)?;
    if !settings.enabled {
        bail!("Enable AI selection in Settings first");
    }
    if settings.base_url.is_empty() || settings.model.is_empty() {
        bail!("Set the Base URL and model first");
    }
    let key = api_key()?;
    let client = http_client()?;
    let total = photos.len();
    let batches = total.div_ceil(MAX_BATCH_SIZE);
    let mut completed = 0usize;

    for (batch_index, chunk) in photos.chunks(MAX_BATCH_SIZE).enumerate() {
        let photo_batch = chunk.to_vec();
        let inputs = tokio::task::spawn_blocking(move || {
            let mut content = Vec::with_capacity(photo_batch.len() * 2);
            for photo in &photo_batch {
                content.extend(photo_input(photo)?);
            }
            Ok::<_, anyhow::Error>(content)
        })
        .await
        .context("AI preview task failed")??;
        let request = json!({
            "model": settings.model,
            "temperature": 0.1,
            "messages": [
                {"role": "system", "content": "You are a photo culling assistant. Evaluate each image for technical quality and photographic value. Return only JSON: {\"results\":[{\"id\":string,\"score\":integer 0..100,\"recommendation\":\"pick\"|\"review\"|\"reject\",\"reasons\":[short strings],\"suggestedRating\":integer 0..5,\"suggestedFlag\":integer -1|0|1}]}. IDs must exactly match input. Do not infer identity, sensitive traits, or facts not visible."},
                {"role": "user", "content": inputs}
            ]
        });

        let result = async {
            let response = client
                .post(format!("{}/chat/completions", settings.base_url))
                .bearer_auth(&key)
                .json(&request)
                .send()
                .await
                .context("AI provider request failed")?;
            let status = response.status();
            let body: Value = response.json().await.context("Provider returned invalid JSON")?;
            if !status.is_success() {
                bail!("Provider returned HTTP {status}");
            }
            parse_results(&body, chunk)
        }
        .await;

        let results = match result {
            Ok(results) => {
                completed += results.len();
                results
            }
            Err(error) => {
                let _ = app.emit(
                    "ai-selection-progress",
                    AiBatchProgress { completed, total, batch: batch_index + 1, batches, results: Vec::new(), error: Some(format!("{error:#}")), finished: true },
                );
                return Err(error);
            }
        };
        let finished = batch_index + 1 == batches;
        let _ = app.emit(
            "ai-selection-progress",
            AiBatchProgress { completed, total, batch: batch_index + 1, batches, results, error: None, finished },
        );
    }
    Ok(())
}

fn http_client() -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(90))
        .connect_timeout(Duration::from_secs(12))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("Unable to initialize HTTP client")
}

fn photo_input(photo: &PhotoGroup) -> Result<Vec<Value>> {
    let path = photo.std_image.as_ref().or(photo.raw_image.as_ref())
        .ok_or_else(|| anyhow!("Photo has no readable image path"))?;
    let image = make_thumbnail_data_url(path)
        .or_else(|_| photo.raw_image.as_deref().filter(|raw| Some(*raw) != photo.std_image.as_deref())
            .ok_or_else(|| anyhow!("No alternative preview available"))
            .and_then(make_thumbnail_data_url))
        .context("Unable to prepare a photo preview for AI")?;
    let content = vec![json!({
        "type": "text",
        "text": format!("Photo id: {}. Filename: {}. Existing rating: {}.", photo.id, photo.base_name, photo.rating)
    }), json!({"type": "image_url", "image_url": {"url": image, "detail": "low"}})];
    Ok(content)
}

fn make_thumbnail_data_url(path: &Path) -> Result<String> {
    let bytes = ImageDecoder::extract_thumbnail_bytes(path)
        .or_else(|_| ImageDecoder::extract_preview_bytes(path))
        .with_context(|| format!("Unable to read preview for {}", path.display()))?;
    let mut reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .context("Unable to identify photo preview format")?;
    reader.limits(image::Limits::default());
    let mut image = reader.decode().context("Unable to decode photo preview")?;
    if image.width().max(image.height()) > MAX_IMAGE_EDGE {
        let ratio = MAX_IMAGE_EDGE as f64 / image.width().max(image.height()) as f64;
        image = image.resize(
            (image.width() as f64 * ratio).round() as u32,
            (image.height() as f64 * ratio).round() as u32,
            FilterType::Triangle,
        );
    }
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, ImageFormat::Jpeg)
        .context("Unable to encode AI preview")?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(output.into_inner());
    Ok(format!("data:image/jpeg;base64,{encoded}"))
}

fn parse_results(body: &Value, photos: &[PhotoGroup]) -> Result<Vec<AiPhotoResult>> {
    let text = body
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Provider response does not contain a text result"))?;
    let content: Value = serde_json::from_str(text).context("AI result was not valid JSON")?;
    let rows = content
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("AI result does not contain a results list"))?;
    let allowed: std::collections::HashSet<&str> = photos.iter().map(|photo| photo.id.as_str()).collect();
    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for row in rows {
        let id = row.get("id").and_then(Value::as_str).ok_or_else(|| anyhow!("AI result has a missing photo id"))?;
        if !allowed.contains(id) {
            bail!("AI returned an unknown photo id");
        }
        if !seen.insert(id) {
            bail!("AI returned a duplicate photo id");
        }
        let score = row.get("score").and_then(Value::as_u64).filter(|score| *score <= 100)
            .ok_or_else(|| anyhow!("AI result has an invalid score"))? as u8;
        let recommendation = row.get("recommendation").and_then(Value::as_str).unwrap_or("review");
        let (suggested_rating, suggested_flag, recommendation) = match recommendation {
            "pick" => (score_to_rating(score), 1, "pick"),
            "reject" => (score_to_rating(score), -1, "reject"),
            _ => (score_to_rating(score), 0, "review"),
        };
        let reasons = row
            .get("reasons")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(Value::as_str).take(4).map(|s| s.chars().take(140).collect()).collect())
            .unwrap_or_default();
        results.push(AiPhotoResult { id: id.to_string(), score, recommendation: recommendation.to_string(), reasons, suggested_rating, suggested_flag });
    }
    if results.len() != photos.len() {
        bail!("AI returned {} of {} photo results", results.len(), photos.len());
    }
    Ok(results)
}

fn score_to_rating(score: u8) -> u8 {
    match score { 0..=19 => 0, 20..=39 => 1, 40..=59 => 2, 60..=74 => 3, 75..=89 => 4, _ => 5 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_mapping_is_bounded_and_monotonic() {
        assert_eq!(score_to_rating(0), 0);
        assert_eq!(score_to_rating(39), 1);
        assert_eq!(score_to_rating(40), 2);
        assert_eq!(score_to_rating(75), 4);
        assert_eq!(score_to_rating(100), 5);
    }
}
