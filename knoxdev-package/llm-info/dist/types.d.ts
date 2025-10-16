export type UseCase = "chat" | "autocomplete";
export type ParameterType = "string" | "number" | "boolean";
export interface Parameter {
    key: string;
    required: boolean;
    valueType: ParameterType;
    displayName?: string;
    description?: string;
    defaultValue?: any;
}
export declare enum ChatTemplate {
    None = "none"
}
export interface LlmInfo {
    model: string;
    displayName?: string;
    description?: string;
    contextLength?: number;
    maxCompletionTokens?: number;
    regex?: RegExp;
    chatTemplate?: ChatTemplate;
    /** If not set, assumes "text" only */
    mediaTypes?: MediaType[];
    recommendedFor?: UseCase[];
    /** Any additional parameters required to configure the model */
    extraParameters?: Parameter[];
}
export type LlmInfoWithProvider = LlmInfo & {
    provider: string;
};
export declare enum MediaType {
    Text = "text",
    Image = "image",
    Audio = "audio",
    Video = "video"
}
export declare const AllMediaTypes: MediaType[];
export interface ApiProviderInfo {
    displayName: string;
    supportsStreaming: boolean;
    handlesTemplating: boolean;
}
export type ModelProviderCapability = "stream" | "fim" | "image" | "template_chat" | "tools";
export interface ModelProvider {
    id: string;
    displayName: string;
    models: Omit<LlmInfo, "provider">[];
    /** Any additional parameters required to configure the model
     *
     * (other than apiKey, apiBase, which are assumed always. And of course model and provider always required)
     */
    extraParameters?: Parameter[];
}
