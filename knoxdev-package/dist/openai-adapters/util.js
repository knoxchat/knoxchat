import { fetchwithRequestOptions } from "../fetch/index.js";
import fetch from "node-fetch";
export function chatChunk(options) {
    return {
        choices: [
            {
                delta: {
                    content: options.content,
                    role: "assistant",
                },
                finish_reason: options.finish_reason ?? "stop",
                index: 0,
                logprobs: null,
            },
        ],
        usage: options.usage,
        created: Date.now(),
        id: options.id ?? "",
        model: options.model,
        object: "chat.completion.chunk",
    };
}
export function chatChunkFromDelta(options) {
    return {
        choices: [
            {
                delta: options.delta,
                finish_reason: options.finish_reason ?? "stop",
                index: 0,
                logprobs: null,
            },
        ],
        usage: options.usage,
        created: Date.now(),
        id: options.id ?? "",
        model: options.model,
        object: "chat.completion.chunk",
    };
}
export function chatCompletion(options) {
    return {
        choices: [
            {
                finish_reason: options.finish_reason ?? "stop",
                index: options.index ?? 0,
                logprobs: null,
                message: {
                    content: options.content ?? null,
                    role: "assistant",
                    refusal: null,
                },
            },
        ],
        usage: options.usage,
        created: Date.now(),
        id: options.id ?? "",
        model: options.model,
        object: "chat.completion",
    };
}
export function model(options) {
    return {
        id: options.id,
        object: "model",
        created: Date.now(),
        owned_by: options.owned_by ?? "organization-owner",
    };
}
export function maybeCustomFetch(requestOptions) {
    return requestOptions
        ? (url, init) => fetchwithRequestOptions(url, init, requestOptions)
        : undefined;
}
export function customFetch(requestOptions) {
    return maybeCustomFetch(requestOptions) ?? fetch;
}
