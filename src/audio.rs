use actix_web::{Error, HttpResponse, web};
use base64::{Engine as _, engine::general_purpose};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::process::Command;

use crate::config;

// ========================================
// Audio Transcription Types
// ========================================

#[derive(Debug, Deserialize)]
pub struct TranscribeAudioRequest {
    pub audio_base64: String,
    pub mime_type: String,
    #[serde(default = "default_agent_for_transcription")]
    pub agent_id: String,
}

fn default_agent_for_transcription() -> String {
    "default".to_string()
}

#[derive(Debug, Serialize)]
pub struct TranscribeAudioResponse {
    pub transcript: String,
}

// ========================================
// Audio Transcription Handler
// ========================================

/// Transcribe audio using Docker-based Whisper
pub async fn transcribe_audio(
    body: web::Json<TranscribeAudioRequest>,
    app_config: web::Data<Arc<config::Config>>,
) -> Result<HttpResponse, Error> {
    debug!("Transcribing audio with Docker Whisper");

    // Validate base64 audio and decode
    let audio_bytes = match general_purpose::STANDARD.decode(&body.audio_base64) {
        Ok(bytes) => bytes,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to decode audio: {}", e)
            })));
        }
    };

    debug!(
        "Audio size: {} bytes, MIME type: {}",
        audio_bytes.len(),
        body.mime_type
    );

    // Use Docker-based Whisper transcription
    match transcribe_with_docker_whisper(&body, audio_bytes, &app_config).await {
        Ok(response) => Ok(response),
        Err(e) => {
            warn!("Docker Whisper transcription failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Transcription failed: {}. Make sure Docker is running and {} is available.", e, app_config.audio.image)
            })))
        }
    }
}

// ========================================
// Docker Whisper Implementation
// ========================================

/// Transcribe audio using Docker with OpenAI Whisper
/// Based on kesertki/whisper:latest Docker image
async fn transcribe_with_docker_whisper(
    _body: &TranscribeAudioRequest,
    audio_bytes: Vec<u8>,
    app_config: &config::Config,
) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Using Docker Whisper for transcription");

    // Create temporary file for audio
    let mut temp_audio_file = NamedTempFile::new()
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    temp_audio_file
        .write_all(&audio_bytes)
        .map_err(|e| format!("Failed to write audio to temp file: {}", e))?;

    let audio_path = temp_audio_file.path();
    let audio_dir = audio_path
        .parent()
        .ok_or("Failed to get audio directory")?;
    let audio_filename = audio_path
        .file_name()
        .ok_or("Failed to get audio filename")?
        .to_str()
        .ok_or("Invalid audio filename")?;

    debug!("Audio saved to: {:?}", audio_path);

    // Get audio configuration from app config (with environment variable fallback)
    let docker_image = std::env::var("SQUID_AUDIO_IMAGE")
        .unwrap_or_else(|_| app_config.audio.image.clone());
    let whisper_model = std::env::var("SQUID_AUDIO_MODEL")
        .unwrap_or_else(|_| app_config.audio.model.clone());

    // Build Docker command with owned strings
    let volume_mount = format!("{}:/app", audio_dir.display());
    let audio_file_path = format!("/app/{}", audio_filename);

    let mut docker_args = vec![
        "run".to_string(),
        "--rm".to_string(),
        "-v".to_string(),
        volume_mount,
        docker_image.clone(),
        audio_file_path,
        "--model".to_string(),
        whisper_model,
        "--output_format".to_string(),
        "json".to_string(),
        "--output_dir".to_string(),
        "/app".to_string(),
        "--task".to_string(),
        "transcribe".to_string(),
    ];

    // Add language if specified (auto-detect by default)
    let language = std::env::var("SQUID_AUDIO_LANGUAGE")
        .unwrap_or_else(|_| app_config.audio.language.clone());

    if !language.is_empty() {
        docker_args.push("--language".to_string());
        docker_args.push(language);
    }

    debug!("Running Docker command: docker {}", docker_args.join(" "));

    // Run Docker command
    let output = Command::new("docker")
        .args(docker_args)
        .output()
        .await
        .map_err(|e| format!("Failed to execute Docker: {}. Is Docker installed?", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Docker Whisper failed: {}", stderr).into());
    }

    // Read the generated JSON file
    let json_filename = audio_path
        .file_stem()
        .ok_or("Failed to get audio file stem")?
        .to_str()
        .ok_or("Invalid file stem")?;
    let json_path = audio_dir.join(format!("{}.json", json_filename));

    debug!("Reading transcription from: {:?}", json_path);

    let json_content = tokio::fs::read_to_string(&json_path)
        .await
        .map_err(|e| format!("Failed to read transcription JSON: {}", e))?;

    // Parse Whisper JSON output
    let whisper_output: Value = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse Whisper JSON: {}", e))?;

    debug!(
        "Whisper output: {}",
        serde_json::to_string_pretty(&whisper_output).unwrap_or_default()
    );

    // Extract transcript text
    let transcript = if let Some(text) = whisper_output.get("text").and_then(|t| t.as_str()) {
        text.trim().to_string()
    } else if whisper_output.is_array() {
        // Handle array format - concatenate all text
        whisper_output
            .as_array()
            .ok_or("Invalid array format")?
            .iter()
            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    } else {
        return Err("Unrecognized JSON format from Whisper".into());
    };

    // Clean up JSON file
    if let Err(e) = tokio::fs::remove_file(&json_path).await {
        debug!("Failed to clean up JSON file: {}", e);
    }

    if transcript.is_empty() {
        return Err("Transcription returned empty result".into());
    }

    debug!(
        "Docker Whisper transcription completed: {} characters",
        transcript.len()
    );

    Ok(HttpResponse::Ok().json(TranscribeAudioResponse { transcript }))
}
