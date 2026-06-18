// nemotron_engine/nemotron_engine.rs
//
// Engine wrapper for the Nemotron streaming RNN-T model: model catalog
// (discover_models), download (HF, with resume/cancel/progress), load/unload,
// and transcription. Mirrors ParakeetEngine for parity; reuses Parakeet's
// DownloadProgress/ModelStatus value types to avoid divergence.

use crate::nemotron_engine::model::NemotronModel;
use crate::parakeet_engine::parakeet_engine::{DownloadProgress, ModelStatus};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::RwLock;
use tokio::time::timeout;

/// The Nemotron model id. fp16, not int8: DirectML silently miscomputes the
/// int8 ConvInteger encoder (output collapses ~35x → always blank), and the CPU
/// EP can't run ConvInteger at all. fp16 has no ConvInteger so it runs correctly
/// on both DirectML (GPU) and CPU. Verified end-to-end vs the ONNX in Python.
pub const NEMOTRON_MODEL: &str = "nemotron-streaming-0.6b-fp16";

const BASE_URL: &str =
    "https://huggingface.co/soniqo/Nemotron-3.5-ASR-Streaming-Multilingual-0.6B-ONNX-FP16/resolve/main";

/// Files to download (sizes from the fp16 repo). Each .onnx graph has a sibling
/// .onnx.data external-weights file that must sit next to it (ort loads it).
const MODEL_FILES: &[(&str, u64)] = &[
    ("encoder.onnx", 22_131_503),
    ("encoder.onnx.data", 1_236_396_032),
    ("decoder.onnx", 7_040),
    ("decoder.onnx.data", 29_880_320),
    ("joint.onnx", 3_207),
    ("joint.onnx.data", 18_911_296),
    ("vocab.json", 236_127),
    ("config.json", 602),
    ("languages.json", 2_020),
];

/// Files that must exist (and pass a min-size check) for a model to be Available.
const REQUIRED_FILES: &[(&str, u64)] = &[
    ("encoder.onnx", 10_000_000),
    ("encoder.onnx.data", 1_100_000_000),
    ("decoder.onnx.data", 25_000_000),
    ("joint.onnx.data", 15_000_000),
    ("vocab.json", 50_000),
    ("languages.json", 500),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: PathBuf,
    pub size_mb: u32,
    pub speed: String,
    pub status: ModelStatus,
    pub description: String,
}

pub struct NemotronEngine {
    models_dir: PathBuf,
    current_model: Arc<RwLock<Option<NemotronModel>>>,
    current_model_name: Arc<RwLock<Option<String>>>,
    pub(crate) available_models: Arc<RwLock<HashMap<String, ModelInfo>>>,
    cancel_download_flag: Arc<RwLock<Option<String>>>,
    pub(crate) active_downloads: Arc<RwLock<HashSet<String>>>,
}

impl NemotronEngine {
    pub fn new_with_models_dir(models_dir: Option<PathBuf>) -> Result<Self> {
        let models_dir = if let Some(dir) = models_dir {
            dir.join("nemotron")
        } else {
            let current_dir = std::env::current_dir()
                .map_err(|e| anyhow!("Failed to get current directory: {}", e))?;
            if cfg!(debug_assertions) {
                current_dir.join("models").join("nemotron")
            } else {
                dirs::data_dir()
                    .or_else(|| dirs::home_dir())
                    .ok_or_else(|| anyhow!("Could not find system data directory"))?
                    .join("Meetily")
                    .join("models")
                    .join("nemotron")
            }
        };

        log::info!("NemotronEngine using models directory: {}", models_dir.display());
        if !models_dir.exists() {
            std::fs::create_dir_all(&models_dir)?;
        }

        Ok(Self {
            models_dir,
            current_model: Arc::new(RwLock::new(None)),
            current_model_name: Arc::new(RwLock::new(None)),
            available_models: Arc::new(RwLock::new(HashMap::new())),
            cancel_download_flag: Arc::new(RwLock::new(None)),
            active_downloads: Arc::new(RwLock::new(HashSet::new())),
        })
    }

    pub async fn discover_models(&self) -> Result<Vec<ModelInfo>> {
        let model_path = self.models_dir.join(NEMOTRON_MODEL);
        let active = self.active_downloads.read().await;

        let status = if active.contains(NEMOTRON_MODEL) {
            ModelStatus::Downloading { progress: 0 }
        } else if model_path.exists() {
            let all_ok = REQUIRED_FILES.iter().all(|(file, min)| {
                std::fs::metadata(model_path.join(file))
                    .map(|m| m.len() >= *min)
                    .unwrap_or(false)
            });
            if all_ok {
                ModelStatus::Available
            } else {
                ModelStatus::Missing
            }
        } else {
            ModelStatus::Missing
        };

        let info = ModelInfo {
            name: NEMOTRON_MODEL.to_string(),
            path: model_path,
            size_mb: 1310,
            speed: "Streaming (FP16)".to_string(),
            status,
            description:
                "NVIDIA Nemotron 3.5 ASR — streaming, multilingual (incl. German). FP16, runs on GPU (DirectML) or CPU. Beta."
                    .to_string(),
        };

        let mut cache = self.available_models.write().await;
        cache.clear();
        cache.insert(info.name.clone(), info.clone());
        Ok(vec![info])
    }

    pub async fn load_model(&self, model_name: &str) -> Result<()> {
        let path = {
            let models = self.available_models.read().await;
            let info = models
                .get(model_name)
                .ok_or_else(|| anyhow!("Nemotron model {} not found", model_name))?;
            if !matches!(info.status, ModelStatus::Available) {
                return Err(anyhow!("Nemotron model {} is not available", model_name));
            }
            info.path.clone()
        };

        if let Some(cur) = self.current_model_name.read().await.as_ref() {
            if cur == model_name {
                return Ok(());
            }
        }
        self.unload_model().await;

        log::info!("Loading Nemotron model: {}", model_name);
        // Always tries DirectML (GPU) then falls back to CPU automatically.
        let model = NemotronModel::new(&path)
            .map_err(|e| anyhow!("Failed to load Nemotron model {}: {}", model_name, e))?;

        *self.current_model.write().await = Some(model);
        *self.current_model_name.write().await = Some(model_name.to_string());
        log::info!("Successfully loaded Nemotron model: {}", model_name);
        Ok(())
    }

    pub async fn unload_model(&self) -> bool {
        let unloaded = self.current_model.write().await.take().is_some();
        self.current_model_name.write().await.take();
        if unloaded {
            log::info!("Nemotron model unloaded");
        }
        unloaded
    }

    pub async fn get_current_model(&self) -> Option<String> {
        self.current_model_name.read().await.clone()
    }

    pub async fn is_model_loaded(&self) -> bool {
        self.current_model.read().await.is_some()
    }

    pub async fn transcribe_audio(
        &self,
        samples: Vec<f32>,
        language: Option<String>,
    ) -> Result<String> {
        let mut guard = self.current_model.write().await;
        let model = guard
            .as_mut()
            .ok_or_else(|| anyhow!("No Nemotron model loaded"))?;
        let slot = model.resolve_lang_slot(language.as_deref());
        model
            .transcribe_samples(samples, slot)
            .map_err(|e| anyhow!("Nemotron transcription failed: {}", e))
    }

    pub async fn cancel_download(&self, model_name: &str) {
        *self.cancel_download_flag.write().await = Some(model_name.to_string());
    }

    /// Download the model from HuggingFace with resume, per-chunk timeout,
    /// cancellation, and weighted progress. Ported from ParakeetEngine.
    pub async fn download_model_detailed(
        &self,
        model_name: &str,
        progress_callback: Option<Box<dyn Fn(DownloadProgress) + Send>>,
    ) -> Result<()> {
        {
            let active = self.active_downloads.read().await;
            if active.contains(model_name) {
                return Err(anyhow!("Download already in progress for {}", model_name));
            }
        }
        self.active_downloads.write().await.insert(model_name.to_string());
        *self.cancel_download_flag.write().await = None;

        // Ensure catalog is populated.
        if self.available_models.read().await.is_empty() {
            let _ = self.discover_models().await;
        }
        let model_dir = self.models_dir.join(model_name);
        if !model_dir.exists() {
            if let Err(e) = fs::create_dir_all(&model_dir).await {
                self.active_downloads.write().await.remove(model_name);
                return Err(anyhow!("Failed to create model directory: {}", e));
            }
        }
        {
            let mut models = self.available_models.write().await;
            if let Some(m) = models.get_mut(model_name) {
                m.status = ModelStatus::Downloading { progress: 0 };
            }
        }

        let client = reqwest::Client::builder()
            .tcp_nodelay(true)
            .pool_max_idle_per_host(1)
            .timeout(Duration::from_secs(3600))
            .connect_timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        let total_size_bytes: u64 = MODEL_FILES.iter().map(|(_, s)| *s).sum();
        let mut already: u64 = 0;
        for (filename, expected) in MODEL_FILES {
            if let Ok(m) = fs::metadata(model_dir.join(filename)).await {
                already += m.len().min(*expected);
            }
        }
        let mut total_downloaded = already;
        let start_time = Instant::now();
        let mut last_report = Instant::now();
        let mut last_pct: u8 = 0;

        for (filename, expected_size) in MODEL_FILES.iter() {
            let file_url = format!("{}/{}", BASE_URL, filename);
            let file_path = model_dir.join(filename);
            let existing_size: u64 = fs::metadata(&file_path).await.map(|m| m.len()).unwrap_or(0);

            // Skip files that already look complete (1% tolerance).
            if *expected_size > 0 && existing_size >= (*expected_size as f64 * 0.99) as u64 {
                continue;
            }

            let mut request = client.get(&file_url);
            if existing_size > 0 {
                request = request.header("Range", format!("bytes={}-", existing_size));
            }
            let response = request
                .send()
                .await
                .map_err(|e| anyhow!("Failed to start download for {}: {}", filename, e))?;

            let resuming = response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
            if !response.status().is_success() && !resuming {
                self.active_downloads.write().await.remove(model_name);
                return Err(anyhow!(
                    "Download failed for {} with status: {}",
                    filename,
                    response.status()
                ));
            }

            let file = if resuming {
                fs::OpenOptions::new().append(true).open(&file_path).await
            } else {
                fs::File::create(&file_path).await
            }
            .map_err(|e| anyhow!("Failed to open {}: {}", filename, e))?;
            let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, file);

            use futures_util::StreamExt;
            let mut stream = response.bytes_stream();
            if !resuming && existing_size > 0 {
                // Server ignored Range; we overwrote, so drop the stale count.
                total_downloaded = total_downloaded.saturating_sub(existing_size.min(*expected_size));
            }

            loop {
                if self.cancel_download_flag.read().await.as_deref() == Some(model_name) {
                    let _ = writer.flush().await;
                    self.active_downloads.write().await.remove(model_name);
                    return Err(anyhow!("Download cancelled by user"));
                }
                match timeout(Duration::from_secs(30), stream.next()).await {
                    Err(_) => {
                        let _ = writer.flush().await;
                        self.active_downloads.write().await.remove(model_name);
                        self.mark_missing(model_name).await;
                        return Err(anyhow!("Download timeout — no data for 30s"));
                    }
                    Ok(None) => break,
                    Ok(Some(Ok(chunk))) => {
                        writer
                            .write_all(&chunk)
                            .await
                            .map_err(|e| anyhow!("Write failed for {}: {}", filename, e))?;
                        total_downloaded += chunk.len() as u64;

                        if last_report.elapsed() >= Duration::from_millis(250) {
                            let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
                            let speed = (total_downloaded.saturating_sub(already)) as f64
                                / 1_048_576.0
                                / elapsed;
                            let prog = DownloadProgress::new(total_downloaded, total_size_bytes, speed);
                            if prog.percent != last_pct {
                                last_pct = prog.percent;
                                if let Some(cb) = &progress_callback {
                                    cb(prog);
                                }
                                self.set_progress(model_name, last_pct).await;
                            }
                            last_report = Instant::now();
                        }
                    }
                    Ok(Some(Err(e))) => {
                        let _ = writer.flush().await;
                        self.active_downloads.write().await.remove(model_name);
                        self.mark_missing(model_name).await;
                        return Err(anyhow!("Download error for {}: {}", filename, e));
                    }
                }
            }
            writer
                .flush()
                .await
                .map_err(|e| anyhow!("Flush failed for {}: {}", filename, e))?;

            // Sanity-check the downloaded file: a Git-LFS pointer or an HTML
            // error page is tiny (a few KB) where we expect MB/GB. Reject it so a
            // flaky CDN response doesn't masquerade as a "downloaded" model.
            if *expected_size > 1_000_000 {
                let got = fs::metadata(&file_path).await.map(|m| m.len()).unwrap_or(0);
                if got < *expected_size / 2 {
                    let _ = fs::remove_file(&file_path).await;
                    self.active_downloads.write().await.remove(model_name);
                    self.mark_missing(model_name).await;
                    return Err(anyhow!(
                        "Downloaded {} is too small ({} bytes, expected ~{}): likely a CDN error or LFS pointer, not the real file",
                        filename, got, expected_size
                    ));
                }
            }
        }

        self.active_downloads.write().await.remove(model_name);
        // Re-validate so status flips to Available.
        let _ = self.discover_models().await;
        if let Some(cb) = &progress_callback {
            cb(DownloadProgress::new(total_size_bytes, total_size_bytes, 0.0));
        }
        log::info!("Nemotron model {} download complete", model_name);
        Ok(())
    }

    async fn set_progress(&self, model_name: &str, pct: u8) {
        let mut models = self.available_models.write().await;
        if let Some(m) = models.get_mut(model_name) {
            m.status = ModelStatus::Downloading { progress: pct };
        }
    }

    async fn mark_missing(&self, model_name: &str) {
        let mut models = self.available_models.write().await;
        if let Some(m) = models.get_mut(model_name) {
            m.status = ModelStatus::Missing;
        }
    }
}
