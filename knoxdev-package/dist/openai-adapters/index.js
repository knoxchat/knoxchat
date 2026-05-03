import dotenv from "dotenv";
import { AnthropicApi } from "./apis/Anthropic.js";
import { MockApi } from "./apis/Mock.js";
import { OpenAIApi } from "./apis/OpenAI.js";
dotenv.config();
function openAICompatible(apiBase, config) {
    return new OpenAIApi({
        ...config,
        apiBase: config.apiBase ?? apiBase,
    });
}
export function constructLlmApi(config) {
    switch (config.provider) {
        case "openai":
            return new OpenAIApi(config);
        case "anthropic":
            return new AnthropicApi(config);
        case "knoxchat":
            return openAICompatible("https://api.knox.chat/v1/", config);
        case "mock":
            return new MockApi();
        default:
            return undefined;
    }
}
