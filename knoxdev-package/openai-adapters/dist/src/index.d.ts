import { BaseLlmApi } from "./apis/base.js";
import { LLMConfig } from "./types.js";
export declare function constructLlmApi(config: LLMConfig): BaseLlmApi | undefined;
export { type ChatCompletion, type ChatCompletionChunk, type ChatCompletionCreateParams, type ChatCompletionCreateParamsNonStreaming, type ChatCompletionCreateParamsStreaming, type Completion, type CompletionCreateParams, type CompletionCreateParamsNonStreaming, type CompletionCreateParamsStreaming, } from "openai/resources/index";
export type { BaseLlmApi } from "./apis/base.js";
export type { LLMConfig } from "./types.js";
