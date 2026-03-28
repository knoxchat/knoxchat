import { LlmInfo, ModelProvider, UseCase } from "./types.js";
export declare const allModelProviders: ModelProvider[];
export declare const allLlms: LlmInfo[];
export declare function findLlmInfo(model: string): LlmInfo | undefined;
export declare function getAllRecommendedFor(useCase: UseCase): LlmInfo[];
