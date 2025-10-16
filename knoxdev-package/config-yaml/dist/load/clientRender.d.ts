import { z } from "zod";
import { PlatformClient } from "../interfaces/index.js";
import { SecretLocation } from "../interfaces/SecretResult.js";
import { PackageSlug } from "../interfaces/slugs.js";
import { AssistantUnrolled } from "../schemas/index.js";
export declare function renderSecrets(packageSlug: PackageSlug, unrolledConfigContent: string, orgScopeId: string | null, // The "scope" that the user is logged in with
onPremProxyUrl: string | null, platformClient?: PlatformClient): Promise<AssistantUnrolled>;
export declare function getUnrenderedSecretLocation(value: string | undefined): SecretLocation | undefined;
export declare function useProxyForUnrenderedSecrets(config: AssistantUnrolled, packageSlug: PackageSlug, orgScopeId: string | null, onPremProxyUrl: string | null): AssistantUnrolled;
/** The additional properties that are added to the otherwise OpenAI-compatible body when requesting a Knox proxy */
export declare const knoxPropertiesSchema: z.ZodObject<{
    apiKeyLocation: z.ZodOptional<z.ZodString>;
    apiBase: z.ZodOptional<z.ZodString>;
    orgScopeId: z.ZodNullable<z.ZodString>;
}, z.core.$strip>;
export type KnoxProperties = z.infer<typeof knoxPropertiesSchema>;
