import { OpenAI } from "openai/index";
import { ChatCompletion, ChatCompletionChunk, ChatCompletionCreateParamsNonStreaming, ChatCompletionCreateParamsStreaming, Completion, CompletionCreateParamsNonStreaming, CompletionCreateParamsStreaming } from "openai/resources/index";
import { AnthropicConfig } from "../types.js";
import { BaseLlmApi, FimCreateParamsStreaming } from "./base.js";
export declare class AnthropicApi implements BaseLlmApi {
    protected config: AnthropicConfig;
    apiBase: string;
    constructor(config: AnthropicConfig);
    private _convertBody;
    private _convertMessages;
    chatCompletionNonStream(body: ChatCompletionCreateParamsNonStreaming, signal: AbortSignal): Promise<ChatCompletion>;
    chatCompletionStream(body: ChatCompletionCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<ChatCompletionChunk, any, unknown>;
    completionNonStream(body: CompletionCreateParamsNonStreaming, signal: AbortSignal): Promise<Completion>;
    completionStream(body: CompletionCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<Completion, any, unknown>;
    fimStream(body: FimCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<ChatCompletionChunk, any, unknown>;
    list(): Promise<OpenAI.Models.Model[]>;
}
