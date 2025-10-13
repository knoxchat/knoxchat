import { Anthropic } from "./providers/anthropic.js";
import { OpenAi } from "./providers/openai.js";
import { LlmInfo, ModelProvider, UseCase } from "./types.js";

export const allModelProviders: ModelProvider[] = [
  OpenAi,
  Anthropic,
];

export const allLlms: LlmInfo[] = allModelProviders.flatMap((provider) =>
  provider.models.map((model) => ({ ...model, provider: provider.id })),
);

export function findLlmInfo(model: string): LlmInfo | undefined {
  return allLlms.find((llm) =>
    llm.regex ? llm.regex.test(model) : llm.model === model,
  );
}

export function getAllRecommendedFor(useCase: UseCase): LlmInfo[] {
  return allLlms.filter((llm) => llm.recommendedFor?.includes(useCase));
}
