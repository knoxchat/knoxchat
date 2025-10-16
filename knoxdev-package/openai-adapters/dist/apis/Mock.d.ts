import { ChatCompletion, ChatCompletionChunk, ChatCompletionCreateParamsNonStreaming, ChatCompletionCreateParamsStreaming, Completion, CompletionCreateParamsNonStreaming, CompletionCreateParamsStreaming, Model } from "openai/resources/index";
import { BaseLlmApi, FimCreateParamsStreaming } from "./base.js";
export declare class MockApi implements BaseLlmApi {
    chatCompletionNonStream(body: ChatCompletionCreateParamsNonStreaming, signal: AbortSignal): Promise<ChatCompletion>;
    chatCompletionStream(body: ChatCompletionCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<ChatCompletionChunk, any, unknown>;
    completionNonStream(body: CompletionCreateParamsNonStreaming, signal: AbortSignal): Promise<Completion>;
    completionStream(body: CompletionCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<Completion, any, unknown>;
    fimStream(body: FimCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<ChatCompletionChunk, any, unknown>;
    list(): Promise<Model[]>;
}
