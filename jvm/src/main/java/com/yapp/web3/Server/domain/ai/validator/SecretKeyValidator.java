package com.yapp.web3.Server.domain.ai.validator;

import com.yapp.web3.Server.global.config.AiSecretKeyProperties;
import com.yapp.web3.Server.global.error.GeneralException;
import com.yapp.web3.Server.global.error.code.status.ErrorStatus;
import lombok.RequiredArgsConstructor;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@Component
@RequiredArgsConstructor
public class SecretKeyValidator {

    @Value("${ai.secret-key}")
    private String configuredSecretKey;

    public void validate(String secretKey) {
        if (secretKey == null || !secretKey.equals(configuredSecretKey)) {
            throw new GeneralException(ErrorStatus.INVALID_SECRET_KEY);
        }
    }
}

