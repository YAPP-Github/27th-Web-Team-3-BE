//! 말투 정제 Few-Shot 예제
//!
//! KIND(상냥체)와 POLITE(정중체) 스타일별 예제입니다.

/// 상냥체 Few-Shot Examples
pub const KIND: &str = r#"# Example 1 (상냥체)
User: "오늘 코딩하다가 존나 빡쳤음 ㅋㅋ 근데 결국 해결함"
Assistant: "오늘 코딩하다가 많이 답답했지만, 결국 문제를 해결할 수 있었어요."

# Example 2 (상냥체)
User: "팀원이랑 싸웠는데 나중에 화해했어 ㅎㅎ"
Assistant: "팀원과 의견 충돌이 있었지만, 이후에 서로 이해하고 화해할 수 있었어요."

# Example 3 (상냥체)
User: "CPO님 CTO님 뭐하세요?? AOS 알람 에러가 생겨서 1시간 동안 50개씩 쌓이고 있는 상황에 CPO 와 CTO가 명확한 전략을 수립해주지 않아 팀의 퍼포먼스 올스탑 되어서 이번 스프린트는 완전히 망해 버린것 같습니다. 성취감도 없고 스트레스도 너무 많이 받은 상태입니다."
Assistant: "AOS 알람 에러가 급증하는 긴박한 상황에서 팀이 기민하게 움직이기 어려웠던 점이 무척 아쉬웠어요. 경영진분들의 명확한 방향 제시가 제때 이루어지지 못해 팀의 노력이 성과로 이어지지 못한 것 같아 다들 마음이 많이 무겁습니다. 다음에는 이런 위기 상황에서도 서로가 믿고 의지하며 빠르게 대처할 수 있도록, 구체적인 대응 전략이 함께 마련된다면 팀원들이 훨씬 더 힘을 낼 수 있을 것 같습니다!""#;

/// 정중체 Few-Shot Examples
pub const POLITE: &str = r#"# Example 1 (정중체)
User: "오늘 코딩하다가 존나 빡쳤음 ㅋㅋ 근데 결국 해결함"
Assistant: "오늘 코딩 중 어려움이 있었으나, 최종적으로 문제를 해결했습니다."

# Example 2 (정중체)
User: "팀원이랑 싸웠는데 나중에 화해했어 ㅎㅎ"
Assistant: "팀원과 의견 차이가 있었으나, 이후 원만하게 해결했습니다."

# Example 3 (정중체)
User: "CPO님 CTO님 뭐하세요?? AOS 알람 에러가 생겨서 1시간 동안 50개씩 쌓이고 있는 상황에 CPO 와 CTO가 명확한 전략을 수립해주지 않아 팀의 퍼포먼스 올스탑 되어서 이번 스프린트는 완전히 망해 버린것 같습니다. 성취감도 없고 스트레스도 너무 많이 받은 상태입니다."
Assistant: "이번 스프린트 중 AOS 알람 에러 이슈 대응 과정에서 의사결정의 공백이 발생하여 팀의 전체적인 업무 효율이 저하되었습니다. 특히 긴급 상황 시 상급 의사결정권자의 명확한 가이드라인 부재로 인해 프로젝트 목표 달성에 차질이 생겼으며, 이로 인해 팀원들이 느끼는 피로도가 높은 상황입니다. 향후 유사 장애 발생 시를 대비한 긴급 전략 수립 프로세스가 보완되기를 희망합니다.""#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_examples_should_have_three_examples() {
        assert!(KIND.contains("Example 1"));
        assert!(KIND.contains("Example 2"));
        assert!(KIND.contains("Example 3"));
    }

    #[test]
    fn polite_examples_should_have_three_examples() {
        assert!(POLITE.contains("Example 1"));
        assert!(POLITE.contains("Example 2"));
        assert!(POLITE.contains("Example 3"));
    }

    #[test]
    fn kind_examples_should_use_kind_style() {
        assert!(KIND.contains("었어요"));
    }

    #[test]
    fn polite_examples_should_use_polite_style() {
        assert!(POLITE.contains("습니다"));
    }

    #[test]
    fn examples_should_have_user_assistant_pairs() {
        assert!(KIND.contains("User:"));
        assert!(KIND.contains("Assistant:"));
        assert!(POLITE.contains("User:"));
        assert!(POLITE.contains("Assistant:"));
    }

    #[test]
    fn example_3_should_handle_complex_cases() {
        assert!(KIND.contains("AOS 알람 에러"));
        assert!(POLITE.contains("AOS 알람 에러"));
    }
}
