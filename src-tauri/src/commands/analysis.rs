use crate::db::Database;
use crate::models::analysis::{AnalysisResult, AnalyzeAudioRequest};
use crate::services::analysis_service;
use tauri::State;

#[tauri::command]
pub fn analyze_audio(
    db: State<'_, Database>,
    request: AnalyzeAudioRequest,
) -> Result<AnalysisResult, String> {
    analysis_service::analyze_audio_file(&db, &request.project_id, &request.file_path)
}

#[tauri::command]
pub fn get_analysis_history(
    db: State<'_, Database>,
) -> Result<Vec<AnalysisResult>, String> {
    analysis_service::get_analysis_history(&db)
}
