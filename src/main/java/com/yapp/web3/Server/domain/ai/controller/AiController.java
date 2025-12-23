package com.yapp.web3.Server.domain.ai.controller;

import com.yapp.web3.Server.domain.ai.dto.request.GuideRequest;
import com.yapp.web3.Server.domain.ai.dto.response.GuideResponse;
import com.yapp.web3.Server.domain.ai.dto.request.RefineRequest;
import com.yapp.web3.Server.domain.ai.dto.response.RefineResponse;
import com.yapp.web3.Server.domain.ai.service.AiService;
import com.yapp.web3.Server.global.common.BaseResponse;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.tags.Tag;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.*;

@Tag(name = "AI", description = "AI 질문 API")
@RestController
@RequestMapping("/api/ai")
@RequiredArgsConstructor
public class AiController {

    private final AiService aiService;

    @Operation(summary = "회고 작성 가이드", description = "작성 중인 회고 내용에 맞춰 AI가 가이드 메시지를 제공합니다.")
    @PostMapping("/retrospective/guide")
    public BaseResponse<GuideResponse> provideGuide(@Valid @RequestBody GuideRequest request) {
        GuideResponse response = aiService.provideGuide(request.getCurrentContent());
        return BaseResponse.onSuccess(response);
    }

    @Operation(summary = "회고 말투 정제", description = "작성된 회고를 AI가 선택한 말투(상냥체(KIND)/공손체(POLITE))로 정제합니다.")
    @PostMapping("/retrospective/refine")
    public BaseResponse<RefineResponse> refineRetrospective(@Valid @RequestBody RefineRequest request) {
        RefineResponse response = aiService.refineRetrospective(request.getContent(), request.getToneStyle());
        return BaseResponse.onSuccess(response);
    }
}

