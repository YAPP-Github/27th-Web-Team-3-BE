//! AI 자동화 파이프라인 모듈
//!
//! Phase 3: AI 코드 수정 시스템을 위한 안전장치 및 제한 기능을 제공합니다.
//!
//! ## 주요 기능
//! - `SafetyChecks`: 코드 수정 전 검증 항목
//! - `FixLimits`: 수정 범위 제한 설정
//! - `FixScope`: 수정 허용/불허 범위 판단

#[allow(dead_code)]
pub mod safety;

#[allow(unused_imports)]
pub use safety::{FixLimits, FixScope, SafetyChecks};
