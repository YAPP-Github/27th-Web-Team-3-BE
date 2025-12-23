package com.yapp.web3.Server.global.error.code.status;

import com.yapp.web3.Server.global.error.code.BaseErrorCode;
import com.yapp.web3.Server.global.error.code.ErrorReasonDTO;
import lombok.AllArgsConstructor;
import lombok.Getter;
import org.springframework.http.HttpStatus;

@Getter
@AllArgsConstructor
public enum ErrorStatus implements BaseErrorCode {
    // 기본 에러
    _INTERNAL_SERVER_ERROR(HttpStatus.INTERNAL_SERVER_ERROR, "COMMON500", "서버 에러, 관리자에게 문의 바랍니다."),
    _BAD_REQUEST(HttpStatus.BAD_REQUEST, "COMMON400", "잘못된 요청입니다."),
    _UNAUTHORIZED(HttpStatus.UNAUTHORIZED, "COMMON401", "인증이 필요합니다."),
    _FORBIDDEN(HttpStatus.FORBIDDEN, "COMMON403", "금지된 요청입니다."),

    // 공통 에러
    PAGE_UNDER_ZERO(HttpStatus.BAD_REQUEST, "COMMON_001", "페이지는 0이상이어야 합니다."),
    MULTIPLE_FIELD_VALIDATION_ERROR(HttpStatus.BAD_REQUEST, "COMMON_002", "입력된 정보에 오류가 있습니다. 필드별 오류 메시지를 참조하세요."),
    NO_MATCHING_ERROR_STATUS(HttpStatus.INTERNAL_SERVER_ERROR, "COMMON_003", "서버 에러. 일치하는 errorStatus를 찾을 수 없습니다."),
    REQUEST_BODY_INVALID(HttpStatus.BAD_REQUEST, "COMMON_004", "요청 본문을 읽을 수 없습니다. 빈 문자열 또는 null이 있는지 확인해주세요."),

    // AI 에러
    INVALID_SECRET_KEY(HttpStatus.UNAUTHORIZED, "AI_001", "유효하지 않은 비밀 키입니다.")
    ;

    private final HttpStatus httpStatus;
    private final String code;
    private final String message;

    @Override
    public ErrorReasonDTO getReason() {
        return ErrorReasonDTO.builder().message(message).code(code).isSuccess(false).build();
    }

    @Override
    public ErrorReasonDTO getReasonHttpStatus() {
        return ErrorReasonDTO.builder()
                .message(message)
                .code(code)
                .isSuccess(false)
                .httpStatus(httpStatus)
                .build();
    }
}
