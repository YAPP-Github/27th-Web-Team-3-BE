package com.yapp.web3.Server.domain.ai.dto.response;

import io.swagger.v3.oas.annotations.media.Schema;
import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Getter;
import lombok.NoArgsConstructor;

@Getter
@Builder
@NoArgsConstructor
@AllArgsConstructor
@Schema(description = "회고 작성 가이드 응답")
public class GuideResponse {

    @Schema(description = "작성 중인 내용", example = "오늘 프로젝트를 진행하면서...")
    private String currentContent;

    @Schema(description = "AI 가이드 메시지", example = "좋은 시작이에요! 구체적으로 어떤 점이 어려웠는지 작성해보면 어떨까요?")
    private String guideMessage;
}

