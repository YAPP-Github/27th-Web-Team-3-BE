package com.yapp.web3.Server.domain.ai.dto.request;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import lombok.Getter;
import lombok.NoArgsConstructor;

@Getter
@NoArgsConstructor
@Schema(description = "회고 작성 가이드 요청")
public class GuideRequest {

    @NotBlank(message = "현재 작성 중인 내용은 필수입니다.")
    @Schema(description = "현재 작성 중인 회고 내용", example = "오늘 프로젝트를 진행하면서...")
    private String currentContent;

    @NotNull (message = "비밀 키는 필수입니다.")
    @Schema(description = "비밀 키", example = "mySecretKey123")
    private String secretKey;
}

