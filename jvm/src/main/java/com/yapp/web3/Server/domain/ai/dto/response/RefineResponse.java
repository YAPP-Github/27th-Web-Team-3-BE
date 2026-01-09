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
@Schema(description = "회고 말투 정제 응답")
public class RefineResponse {

    @Schema(description = "원본 내용", example = "오늘 일 존나 힘들었음 ㅋㅋ 근데 배운게 많았어")
    private String originalContent;

    @Schema(description = "정제된 내용", example = "오늘 업무가 힘들었지만, 그만큼 많은 것을 배울 수 있었어요.")
    private String refinedContent;

    @Schema(description = "적용된 말투 스타일", example = "KIND")
    private String toneStyle;
}

