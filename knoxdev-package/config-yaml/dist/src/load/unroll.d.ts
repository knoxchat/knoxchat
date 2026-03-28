import { PlatformClient, Registry } from "../interfaces/index.js";
import { FullSlug } from "../interfaces/slugs.js";
import { AssistantUnrolled, Block, ConfigYaml } from "../schemas/index.js";
export declare function parseConfigYaml(configYaml: string): ConfigYaml;
export declare function parseAssistantUnrolled(configYaml: string): AssistantUnrolled;
export declare function parseBlock(configYaml: string): Block;
export declare function getTemplateVariables(templatedYaml: string): string[];
export declare function fillTemplateVariables(templatedYaml: string, data: {
    [key: string]: string;
}): string;
export interface TemplateData {
    inputs: Record<string, string> | undefined;
    secrets: Record<string, string> | undefined;
    knox: {};
}
export interface DoNotRenderSecretsUnrollAssistantOptions {
    renderSecrets: false;
}
export interface RenderSecretsUnrollAssistantOptions {
    renderSecrets: true;
    orgScopeId: string | null;
    currentUserSlug: string;
    platformClient: PlatformClient;
    onPremProxyUrl: string | null;
}
export type UnrollAssistantOptions = DoNotRenderSecretsUnrollAssistantOptions | RenderSecretsUnrollAssistantOptions;
export declare function unrollAssistant(fullSlug: string, registry: Registry, options: UnrollAssistantOptions): Promise<AssistantUnrolled>;
export declare function unrollAssistantFromContent(assistantSlug: FullSlug, rawYaml: string, registry: Registry, options: UnrollAssistantOptions): Promise<AssistantUnrolled>;
export declare function unrollBlocks(assistant: ConfigYaml, registry: Registry): Promise<AssistantUnrolled>;
export declare function resolveBlock(fullSlug: FullSlug, inputs: Record<string, string | unknown> | undefined, registry: Registry): Promise<AssistantUnrolled>;
export declare function mergeOverrides<T extends Record<string, any>>(block: T, overrides: Partial<T>): T;
