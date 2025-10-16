import { RequestOptions } from "knoxdev-knox-config-types";
import fetch from "node-fetch";
import { ChatCompletionChunk, ChatCompletion, CompletionUsage, Model } from "openai/resources/index";
export declare function chatChunk(options: {
    content: string | null | undefined;
    model: string;
    finish_reason?: ChatCompletionChunk.Choice["finish_reason"];
    id?: string | null;
    usage?: CompletionUsage;
}): ChatCompletionChunk;
export declare function chatChunkFromDelta(options: {
    delta: ChatCompletionChunk.Choice["delta"];
    model: string;
    finish_reason?: ChatCompletionChunk.Choice["finish_reason"];
    id?: string | null;
    usage?: CompletionUsage;
}): ChatCompletionChunk;
export declare function chatCompletion(options: {
    content: string | null | undefined;
    model: string;
    finish_reason?: ChatCompletion.Choice["finish_reason"];
    id?: string | null;
    usage?: CompletionUsage;
    index?: number | null;
}): ChatCompletion;
export declare function model(options: {
    id: string;
    owned_by?: string;
}): Model;
export declare function maybeCustomFetch(requestOptions: RequestOptions | undefined): ((url: any, init: any) => Promise<import("node-fetch").Response>) | undefined;
export declare function customFetch(requestOptions: RequestOptions | undefined): typeof fetch;
