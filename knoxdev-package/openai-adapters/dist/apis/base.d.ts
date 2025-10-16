import { ChatCompletion, ChatCompletionChunk, ChatCompletionCreateParamsNonStreaming, ChatCompletionCreateParamsStreaming, Completion, CompletionCreateParamsNonStreaming, CompletionCreateParamsStreaming, Model } from "openai/resources/index";
export interface FimCreateParamsStreaming extends CompletionCreateParamsStreaming {
    suffix: string;
}
export interface BaseLlmApi {
    chatCompletionNonStream(body: ChatCompletionCreateParamsNonStreaming, signal: AbortSignal): Promise<ChatCompletion>;
    chatCompletionStream(body: ChatCompletionCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<ChatCompletionChunk>;
    completionNonStream(body: CompletionCreateParamsNonStreaming, signal: AbortSignal): Promise<Completion>;
    completionStream(body: CompletionCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<Completion>;
    fimStream(body: FimCreateParamsStreaming, signal: AbortSignal): AsyncGenerator<ChatCompletionChunk>;
    list(): Promise<Model[]>;
}
