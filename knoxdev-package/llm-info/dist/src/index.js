import { Anthropic } from "./providers/anthropic.js";
import { OpenAi } from "./providers/openai.js";
export const allModelProviders = [
    OpenAi,
    Anthropic,
];
export const allLlms = allModelProviders.flatMap((provider) => provider.models.map((model) => ({ ...model, provider: provider.id })));
export function findLlmInfo(model) {
    return allLlms.find((llm) => llm.regex ? llm.regex.test(model) : llm.model === model);
}
export function getAllRecommendedFor(useCase) {
    return allLlms.filter((llm) => llm.recommendedFor?.includes(useCase));
}
