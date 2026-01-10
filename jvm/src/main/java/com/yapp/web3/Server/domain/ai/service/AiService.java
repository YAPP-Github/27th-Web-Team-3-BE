package com.yapp.web3.Server.domain.ai.service;

import com.yapp.web3.Server.domain.ai.dto.response.GuideResponse;
import com.yapp.web3.Server.domain.ai.dto.request.RefineRequest.ToneStyle;
import com.yapp.web3.Server.domain.ai.dto.response.RefineResponse;
import com.yapp.web3.Server.domain.ai.prompt.PromptTemplate;
import com.yapp.web3.Server.domain.ai.validator.SecretKeyValidator;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.ai.chat.model.ChatModel;
import org.springframework.ai.chat.prompt.Prompt;
import org.springframework.ai.chat.messages.Message;
import org.springframework.ai.chat.messages.SystemMessage;
import org.springframework.ai.chat.messages.UserMessage;
import org.springframework.stereotype.Service;

import java.util.List;

@Slf4j
@Service
@RequiredArgsConstructor
public class AiService {

    private final ChatModel chatModel;
    private final SecretKeyValidator secretKeyValidator;

    public GuideResponse provideGuide(String currentContent, String secretKey) {
        // Secret Key 검증
        secretKeyValidator.validate(secretKey);

        List<Message> messages = List.of(
                new SystemMessage(PromptTemplate.GUIDE_SYSTEM_PROMPT),
                new UserMessage(PromptTemplate.GUIDE_FEW_SHOT_EXAMPLES),
                new UserMessage("User: \"" + currentContent + "\"")
        );

        Prompt prompt = new Prompt(messages);
        String guideMessage = chatModel.call(prompt).getResult().getOutput().getText();

        return GuideResponse.builder()
                .currentContent(currentContent)
                .guideMessage(guideMessage)
                .build();
    }

    public RefineResponse refineRetrospective(String content, ToneStyle toneStyle, String secretKey) {
        // Secret Key 검증
        secretKeyValidator.validate(secretKey);

        String systemPrompt = PromptTemplate.getRefineSystemPrompt(toneStyle);
        String fewShotExamples = PromptTemplate.getRefineFewShotExamples(toneStyle);

        List<Message> messages = List.of(
                new SystemMessage(systemPrompt),
                new UserMessage(fewShotExamples),
                new UserMessage("User: \"" + content + "\"")
        );

        Prompt prompt = new Prompt(messages);
        String refinedContent = chatModel.call(prompt).getResult().getOutput().getText();

        // "Assistant: " 접두사 제거 (있는 경우)
        refinedContent = refinedContent.replaceFirst("^Assistant:\\s*", "").trim();

        // 따옴표 제거 (있는 경우)
        refinedContent = refinedContent.replaceAll("^\"|\"$", "").trim();

        return RefineResponse.builder()
                .originalContent(content)
                .refinedContent(refinedContent)
                .toneStyle(toneStyle.name())
                .build();
    }
}

