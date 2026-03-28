import { streamSse } from "knoxdev-knox-fetch";
import { OpenAI } from "openai/index";
import { customFetch } from "../util.js";
export class OpenAIApi {
    config;
    openai;
    apiBase = "https://api.openai.com/v1/";
    constructor(config) {
        this.config = config;
        this.apiBase = config.apiBase ?? this.apiBase;
        this.openai = new OpenAI({
            apiKey: config.apiKey,
            baseURL: this.apiBase,
            fetch: customFetch(config.requestOptions),
        });
    }
    modifyChatBody(body) {
        // o-series models
        if (body.model.startsWith("o")) {
            // a) use max_completion_tokens instead of max_tokens
            body.max_completion_tokens = body.max_tokens;
            body.max_tokens = undefined;
            // b) use "developer" message role rather than "system"
            body.messages = body.messages.map((message) => {
                if (message.role === "system") {
                    return { ...message, role: "developer" };
                }
                return message;
            });
        }
        return body;
    }
    async chatCompletionNonStream(body, signal) {
        const response = await this.openai.chat.completions.create(this.modifyChatBody(body), {
            signal,
        });
        return response;
    }
    async *chatCompletionStream(body, signal) {
        const response = await this.openai.chat.completions.create(this.modifyChatBody(body), {
            signal,
        });
        for await (const result of response) {
            yield result;
        }
    }
    async completionNonStream(body, signal) {
        const response = await this.openai.completions.create(body, { signal });
        return response;
    }
    async *completionStream(body, signal) {
        const response = await this.openai.completions.create(body, { signal });
        for await (const result of response) {
            yield result;
        }
    }
    async *fimStream(body, signal) {
        const endpoint = new URL("fim/completions", this.apiBase);
        const resp = await customFetch(this.config.requestOptions)(endpoint, {
            method: "POST",
            body: JSON.stringify({
                model: body.model,
                prompt: body.prompt,
                suffix: body.suffix,
                max_tokens: body.max_tokens,
                max_completion_tokens: body.max_completion_tokens,
                temperature: body.temperature,
                top_p: body.top_p,
                frequency_penalty: body.frequency_penalty,
                presence_penalty: body.presence_penalty,
                stop: body.stop,
                stream: true,
            }),
            headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
                "x-api-key": this.config.apiKey ?? "",
                Authorization: `Bearer ${this.config.apiKey}`,
            },
            signal,
        });
        for await (const chunk of streamSse(resp)) {
            if (chunk.choices && chunk.choices.length > 0) {
                yield chunk;
            }
        }
    }
    async list() {
        return (await this.openai.models.list()).data;
    }
}
