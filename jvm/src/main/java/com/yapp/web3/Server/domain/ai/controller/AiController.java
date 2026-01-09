package com.yapp.web3.Server.domain.ai.controller;

import com.yapp.web3.Server.domain.ai.dto.request.GuideRequest;
import com.yapp.web3.Server.domain.ai.dto.response.GuideResponse;
import com.yapp.web3.Server.domain.ai.dto.request.RefineRequest;
import com.yapp.web3.Server.domain.ai.dto.response.RefineResponse;
import com.yapp.web3.Server.domain.ai.service.AiService;
import com.yapp.web3.Server.global.common.BaseResponse;
import io.swagger.v3.oas.annotations.Operation;
import io.swagger.v3.oas.annotations.media.Content;
import io.swagger.v3.oas.annotations.media.ExampleObject;
import io.swagger.v3.oas.annotations.media.Schema;
import io.swagger.v3.oas.annotations.responses.ApiResponse;
import io.swagger.v3.oas.annotations.responses.ApiResponses;
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
    @ApiResponses(value = {
            @ApiResponse(
                    responseCode = "200",
                    description = "성공",
                    content = @Content(
                            mediaType = "application/json",
                            schema = @Schema(implementation = BaseResponse.class),
                            examples = @ExampleObject(value = """
                                    {
                                      "isSuccess": true,
                                      "code": "COMMON200",
                                      "message": "성공입니다.",
                                      "result": {
                                        "currentContent": "오늘 프로젝트를 진행하면서...",
                                        "guideMessage": "좋은 시작이에요! 프로젝트 진행 과정에서 어떤 점이 특히 기억에 남으시나요? 구체적인 경험을 추가하면 더 의미 있는 회고가 될 거예요."
                                      }
                                    }
                                    """)
                    )
            ),
            @ApiResponse(
                    responseCode = "400",
                    description = "잘못된 요청 - 필수 값 누락",
                    content = @Content(
                            mediaType = "application/json",
                            examples = @ExampleObject(value = """
                                    {
                                      "isSuccess": false,
                                      "code": "COMMON400",
                                      "message": "잘못된 요청입니다.",
                                      "result": null
                                    }
                                    """)
                    )
            ),
            @ApiResponse(
                    responseCode = "401",
                    description = "인증 실패 - 유효하지 않은 비밀 키",
                    content = @Content(
                            mediaType = "application/json",
                            examples = @ExampleObject(value = """
                                    {
                                      "isSuccess": false,
                                      "code": "AI_001",
                                      "message": "유효하지 않은 비밀 키입니다.",
                                      "result": null
                                    }
                                    """)
                    )
            ),
            @ApiResponse(
                    responseCode = "500",
                    description = "서버 에러",
                    content = @Content(
                            mediaType = "application/json",
                            examples = @ExampleObject(value = """
                                    {
                                      "isSuccess": false,
                                      "code": "COMMON500",
                                      "message": "서버 에러, 관리자에게 문의 바랍니다.",
                                      "result": null
                                    }
                                    """)
                    )
            )
    })
    @PostMapping("/retrospective/guide")
    public BaseResponse<GuideResponse> provideGuide(@Valid @RequestBody GuideRequest request) {
        GuideResponse response = aiService.provideGuide(request.getCurrentContent(), request.getSecretKey());
        return BaseResponse.onSuccess(response);
    }

    @Operation(summary = "회고 말투 정제", description = "작성된 회고를 AI가 선택한 말투(상냥체(KIND)/정중체(POLITE))로 정제합니다.")
    @ApiResponses(value = {
            @ApiResponse(
                    responseCode = "200",
                    description = "성공",
                    content = @Content(
                            mediaType = "application/json",
                            schema = @Schema(implementation = BaseResponse.class),
                            examples = @ExampleObject(value = """
                                    {
                                      "isSuccess": true,
                                      "code": "COMMON200",
                                      "message": "성공입니다.",
                                      "result": {
                                        "originalContent": "오늘 일 존나 힘들었음 ㅋㅋ 근데 배운게 많았어",
                                        "refinedContent": "오늘 일이 많이 힘들었어요. 하지만 배운 것이 정말 많았어요.",
                                        "toneStyle": "KIND"
                                      }
                                    }
                                    """)
                    )
            ),
            @ApiResponse(
                    responseCode = "400",
                    description = "잘못된 요청 - 필수 값 누락 또는 잘못된 enum 값",
                    content = @Content(
                            mediaType = "application/json",
                            examples = {
                                    @ExampleObject(
                                            name = "필수 값 누락",
                                            value = """
                                                    {
                                                      "isSuccess": false,
                                                      "code": "COMMON400",
                                                      "message": "잘못된 요청입니다.",
                                                      "result": null
                                                    }
                                                    """
                                    ),
                                    @ExampleObject(
                                            name = "유효하지 않은 말투 스타일",
                                            value = """
                                                    {
                                                      "isSuccess": false,
                                                      "code": "AI_002",
                                                      "message": "유효하지 않은 말투 스타일입니다. KIND 또는 POLITE만 가능합니다.",
                                                      "result": null
                                                    }
                                                    """
                                    )
                            }
                    )
            ),
            @ApiResponse(
                    responseCode = "401",
                    description = "인증 실패 - 유효하지 않은 비밀 키",
                    content = @Content(
                            mediaType = "application/json",
                            examples = @ExampleObject(value = """
                                    {
                                      "isSuccess": false,
                                      "code": "AI_001",
                                      "message": "유효하지 않은 비밀 키입니다.",
                                      "result": null
                                    }
                                    """)
                    )
            ),
            @ApiResponse(
                    responseCode = "500",
                    description = "서버 에러",
                    content = @Content(
                            mediaType = "application/json",
                            examples = @ExampleObject(value = """
                                    {
                                      "isSuccess": false,
                                      "code": "COMMON500",
                                      "message": "서버 에러, 관리자에게 문의 바랍니다.",
                                      "result": null
                                    }
                                    """)
                    )
            )
    })
    @PostMapping("/retrospective/refine")
    public BaseResponse<RefineResponse> refineRetrospective(@Valid @RequestBody RefineRequest request) {
        RefineResponse response = aiService.refineRetrospective(request.getContent(), request.getToneStyle(), request.getSecretKey());
        return BaseResponse.onSuccess(response);
    }
}

