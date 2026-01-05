use crate::LedController;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub leds: Arc<LedController>,
}

#[derive(Serialize, Deserialize)]
pub struct LedResponse {
    pub led: u8,
    pub state: String, // "on", "off", or "blinking"
}

#[derive(Serialize, Deserialize)]
pub struct BlinkRequest {
    pub frequency_ms: u64,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub message: String,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/api/leds", get(get_all_leds))
        .route("/api/leds/:led", get(get_led))
        .route("/api/leds/:led/on", post(set_led_on))
        .route("/api/leds/:led/off", post(set_led_off))
        .route("/api/leds/:led/blink", post(set_led_blink))
        .route("/api/leds/all/off", post(set_all_leds_off))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn root() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok".to_string(),
        message: "Train Set Control API - LED Control Only".to_string(),
    })
}

// LED endpoints
async fn get_all_leds(State(state): State<AppState>) -> Result<Json<Vec<LedResponse>>, StatusCode> {
    let mut leds = Vec::new();
    for led_num in 1..=24 {
        leds.push(LedResponse {
            led: led_num,
            state: "unknown".to_string(), // We don't track state
        });
    }
    Ok(Json(leds))
}

async fn get_led(
    State(_state): State<AppState>,
    Path(led): Path<u8>,
) -> Result<Json<LedResponse>, StatusCode> {
    if led < 1 || led > 24 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(LedResponse {
        led,
        state: "unknown".to_string(), // We don't track state
    }))
}

async fn set_led_on(
    State(state): State<AppState>,
    Path(led): Path<u8>,
) -> Result<Json<StatusResponse>, StatusCode> {
    if led < 1 || led > 24 {
        return Err(StatusCode::NOT_FOUND);
    }
    state.leds.on(led).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(StatusResponse {
        status: "ok".to_string(),
        message: format!("LED {} turned on", led),
    }))
}

async fn set_led_off(
    State(state): State<AppState>,
    Path(led): Path<u8>,
) -> Result<Json<StatusResponse>, StatusCode> {
    if led < 1 || led > 24 {
        return Err(StatusCode::NOT_FOUND);
    }
    state.leds.off(led).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(StatusResponse {
        status: "ok".to_string(),
        message: format!("LED {} turned off", led),
    }))
}

async fn set_led_blink(
    State(state): State<AppState>,
    Path(led): Path<u8>,
    Json(request): Json<BlinkRequest>,
) -> Result<Json<StatusResponse>, StatusCode> {
    if led < 1 || led > 24 {
        return Err(StatusCode::NOT_FOUND);
    }
    if request.frequency_ms == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }
    state.leds.blink(led, request.frequency_ms).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(StatusResponse {
        status: "ok".to_string(),
        message: format!("LED {} blinking at {}ms interval", led, request.frequency_ms),
    }))
}

async fn set_all_leds_off(State(state): State<AppState>) -> Result<Json<StatusResponse>, StatusCode> {
    state.leds.all_off().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(StatusResponse {
        status: "ok".to_string(),
        message: "All LEDs turned off and blinking cancelled".to_string(),
    }))
}
