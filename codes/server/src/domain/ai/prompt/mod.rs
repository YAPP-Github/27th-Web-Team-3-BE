//! AI 프롬프트 모듈
//!
//! 회고 작성 가이드와 말투 정제에 사용되는 프롬프트를 관리합니다.
//!
//! ## 구조
//! - `guide`: 회고 작성 가이드 프롬프트
//! - `refine`: 말투 정제 프롬프트
//! - `examples`: Few-shot 예제

mod examples;
mod guide;
mod refine;

// 가이드 프롬프트 재노출
pub use guide::FEW_SHOT_EXAMPLES as GUIDE_FEW_SHOT_EXAMPLES;
pub use guide::SYSTEM_PROMPT as GUIDE_SYSTEM_PROMPT;

// 정제 프롬프트 재노출
pub use refine::few_shot_examples as get_refine_few_shot_examples;
pub use refine::system_prompt as get_refine_system_prompt;
