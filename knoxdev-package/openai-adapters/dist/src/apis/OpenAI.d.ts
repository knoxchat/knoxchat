import { OpenAI } from "openai/index";
import { ChatCompletion, ChatCompletionChunk, ChatCompletionCreateParams, ChatCompletionCreateParamsNonStreaming, ChatCompletionCreateParamsStreaming, Completion, CompletionCreateParamsNonStreaming, CompletionCreateParamsStreaming, Model } from "openai/resources/index";
import { z } from "zod";
import { OpenAIConfigSchema } from "../types.js";
import { BaseLlmApi, FimCreateParamsStreaming } from "./base.js";
export declare class OpenAIApi implements BaseLlmApi {
    protected config: z.infer<typeof OpenAIConfigSchema>;
    openai: OpenAI;
    apiBase: string;
    constructor(config: z.infer<typeof OpenAIConfigSchema>);
    modifyChatBody<T extends ChatCompletionCreateParams>(body: T): T;
    chatCompletionNonStream(body: ChatCompletionCreateParamsNonStreaming, signal: AbortSignal): Promise<ChatCompletion>;
    chatCompletionStream(body: ChatCompletionCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<ChatCompletionChunk, any, unknown>;
    completionNonStream(body: CompletionCreateParamsNonStreaming, signal: AbortSignal): Promise<Completion>;
    completionStream(body: CompletionCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<Completion, any, unknown>;
    fimStream(body: FimCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<ChatCompletionChunk, any, unknown>;
    list(): Promise<Model[]>;
}
