package com.yapp.web3.Server.domain.ai.dto.request;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import lombok.Getter;
import lombok.NoArgsConstructor;

@Getter
@NoArgsConstructor
@Schema(description = "회고 말투 정제 요청")
public class RefineRequest {

    @NotBlank(message = "회고 내용은 필수입니다.")
    @Schema(description = "정제할 회고 내용", example = "오늘 일 존나 힘들었음 ㅋㅋ 근데 배운게 많았어")
    private String content;

    @NotNull(message = "말투 스타일은 필수입니다.")
    @Schema(description = "말투 스타일", example = "KIND", allowableValues = {"KIND", "POLITE", "PROFESSIONAL"})
    private ToneStyle toneStyle;

    public enum ToneStyle {
        KIND,         // 상냥체
        POLITE       // 공손체
    }
}

