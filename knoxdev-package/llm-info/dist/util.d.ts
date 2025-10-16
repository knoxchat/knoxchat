import { LlmInfo } from "./types.js";
export declare function llms(provider: string, infos: Omit<LlmInfo, "provider">[]): LlmInfo[];
