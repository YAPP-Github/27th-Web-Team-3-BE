package com.yapp.web3.Server.global.error.code;

public interface BaseErrorCode {

    ErrorReasonDTO getReason();

    ErrorReasonDTO getReasonHttpStatus();
}
