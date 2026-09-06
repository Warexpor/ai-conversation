use crate::engine::{
    build_chat_body, chat_url, extract_sse_deltas, extract_text_content, models_url,
    parse_models_response, StreamPiece,
};
use crate::state::{AiConfig, Message};
use futures_util::StreamExt;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

/// Non-streaming call. Returns (content, optional reasoning).
pub async fn call_llm(
    config: &AiConfig,
    speaking_agent: &str,
    messages_context: &[Message],
    narration: Option<&str>,
) -> Result<(String, Option<String>), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let body = build_chat_body(config, speaking_agent, messages_context, false, narration);
    let url = chat_url(&config.api_base_url);

    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let status = res.status();
    let data: Value = res
        .json()
        .await
        .map_err(|e| format!("Parse failed: {}", e))?;

    if !status.is_success() {
        let err_msg = data["error"]["message"]
            .as_str()
            .or_else(|| data["error"].as_str())
            .unwrap_or("unknown API error");
        return Err(format!("API {}: {}", status, err_msg));
    }

    let msg = &data["choices"][0]["message"];
    let finish_reason = data["choices"][0]["finish_reason"].as_str().unwrap_or("stop");

    let (content, reasoning): (String, Option<String>) = match extract_text_content(&msg["content"]) {
        Some(text) => {
            let mut reasoning = None;
            for key in ["reasoning_content", "reasoning", "thinking", "reasoning_text"] {
                if let Some(t) = extract_text_content(&msg[key]) {
                    reasoning = Some(t);
                    break;
                }
            }
            (text, reasoning)
        }
        None => {
            // Content empty — try reasoning as fallback visible text
            if let Some(r) = extract_text_content(&msg["reasoning_content"])
                .or_else(|| extract_text_content(&msg["reasoning"]))
                .or_else(|| extract_text_content(&msg["thinking"]))
            {
                // Reasoning models sometimes put output in reasoning fields
                (r.clone(), Some(r))
            } else if let Some(t) = extract_text_content(&data["choices"][0]["text"]) {
                (t, None)
            } else {
                return Err(format!(
                    "No content in API response (finish_reason: {})",
                    finish_reason
                ));
            }
        }
    };

    Ok((content, reasoning))
}

fn emit_chunk(
    app_handle: &AppHandle,
    agent: &str,
    turn: u32,
    kind: &str,
    delta: &str,
) {
    if delta.is_empty() {
        return;
    }
    let _ = app_handle.emit(
        "stream-chunk",
        serde_json::json!({
            "agent": agent,
            "turn": turn,
            "kind": kind,
            "delta": delta,
        }),
    );
}

/// Stream chat completions via SSE.
/// Returns (answer content, optional full reasoning text).
pub async fn stream_llm(
    config: &AiConfig,
    speaking_agent: &str,
    messages_context: &[Message],
    narration: Option<&str>,
    turn: u32,
    app_handle: &AppHandle,
) -> Result<(String, Option<String>), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let body = build_chat_body(config, speaking_agent, messages_context, true, narration);
    let url = chat_url(&config.api_base_url);

    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Stream request failed: {}", e))?;

    let status = res.status();
    if !status.is_success() {
        let data: Value = res.json().await.unwrap_or(serde_json::json!({}));
        let err_msg = data["error"]["message"]
            .as_str()
            .or_else(|| data["error"].as_str())
            .unwrap_or("unknown API error");
        return Err(format!("API {}: {}", status, err_msg));
    }

    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    if content_type.contains("application/json") && !content_type.contains("event-stream") {
        let data: Value = res
            .json()
            .await
            .map_err(|e| format!("Parse failed: {}", e))?;
        let msg = &data["choices"][0]["message"];
        let finish_reason = data["choices"][0]["finish_reason"].as_str().unwrap_or("stop");

        let (text, reasoning): (String, Option<String>) = match extract_text_content(&msg["content"]) {
            Some(c) => {
                let mut r = None;
                for key in ["reasoning_content", "reasoning", "thinking", "reasoning_text"] {
                    if let Some(t) = extract_text_content(&msg[key]) {
                        r = Some(t);
                        break;
                    }
                }
                (c, r)
            }
            None => {
                if let Some(r) = extract_text_content(&msg["reasoning_content"])
                    .or_else(|| extract_text_content(&msg["reasoning"]))
                    .or_else(|| extract_text_content(&msg["thinking"]))
                {
                    (r.clone(), Some(r))
                } else if let Some(t) = extract_text_content(&data["choices"][0]["text"]) {
                    (t, None)
                } else {
                    return Err(format!(
                        "No content in API response (finish_reason: {})",
                        finish_reason
                    ));
                }
            }
        };

        let _ = app_handle.emit(
            "stream-start",
            serde_json::json!({ "agent": speaking_agent, "turn": turn }),
        );
        if let Some(ref r) = reasoning {
            emit_chunk(app_handle, speaking_agent, turn, "reasoning", r);
        }
        emit_chunk(app_handle, speaking_agent, turn, "content", &text);
        return Ok((text, reasoning));
    }

    let _ = app_handle.emit(
        "stream-start",
        serde_json::json!({
            "agent": speaking_agent,
            "turn": turn,
        }),
    );

    let mut stream = res.bytes_stream();
    let mut raw: Vec<u8> = Vec::new();
    let mut text_buf = String::new();
    let mut full = String::new();
    let mut full_reasoning = String::new();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Stream read failed: {}", e))?;
        raw.extend_from_slice(&chunk);

        match std::str::from_utf8(&raw) {
            Ok(s) => {
                text_buf.push_str(s);
                raw.clear();
            }
            Err(e) => {
                let valid_up_to = e.valid_up_to();
                if valid_up_to > 0 {
                    let ok = std::str::from_utf8(&raw[..valid_up_to]).unwrap();
                    text_buf.push_str(ok);
                    raw.drain(..valid_up_to);
                }
            }
        }

        let normalized = text_buf.replace("\r\n", "\n").replace('\r', "\n");
        let (deltas, rest) = extract_sse_deltas(&normalized);
        text_buf = rest;
        for piece in deltas {
            match piece {
                StreamPiece::Content(d) => {
                    full.push_str(&d);
                    emit_chunk(app_handle, speaking_agent, turn, "content", &d);
                }
                StreamPiece::Reasoning(d) => {
                    full_reasoning.push_str(&d);
                    emit_chunk(app_handle, speaking_agent, turn, "reasoning", &d);
                }
            }
        }
    }

    if !text_buf.is_empty() {
        let (deltas, _) = extract_sse_deltas(&(text_buf + "\n\n"));
        for piece in deltas {
            match piece {
                StreamPiece::Content(d) => {
                    full.push_str(&d);
                    emit_chunk(app_handle, speaking_agent, turn, "content", &d);
                }
                StreamPiece::Reasoning(d) => {
                    full_reasoning.push_str(&d);
                    emit_chunk(app_handle, speaking_agent, turn, "reasoning", &d);
                }
            }
        }
    }

    if full.is_empty() {
        let _ = app_handle.emit(
            "stream-abort",
            serde_json::json!({ "agent": speaking_agent, "turn": turn }),
        );
        return call_llm(config, speaking_agent, messages_context, narration).await;
    }

    let reasoning = if full_reasoning.is_empty() {
        None
    } else {
        Some(full_reasoning)
    };
    Ok((full, reasoning))
}

pub async fn fetch_models(base_url: &str, api_key: &str) -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let url = models_url(base_url);
    let mut req = client.get(&url);
    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    let res = req
        .send()
        .await
        .map_err(|e| format!("Models request failed: {}", e))?;

    let status = res.status();
    let data: Value = res
        .json()
        .await
        .map_err(|e| format!("Models parse failed: {}", e))?;

    if !status.is_success() {
        let err_msg = data["error"]["message"]
            .as_str()
            .or_else(|| data["error"].as_str())
            .unwrap_or("unknown API error");
        return Err(format!("Models API {}: {}", status, err_msg));
    }

    parse_models_response(&data)
}
