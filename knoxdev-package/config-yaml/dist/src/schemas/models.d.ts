import { z } from "zod";
export declare const clientCertificateOptionsSchema: z.ZodObject<{
    cert: z.ZodString;
    key: z.ZodString;
    passphrase: z.ZodOptional<z.ZodString>;
}, z.core.$strip>;
export type ClientCertificateOptions = z.infer<typeof clientCertificateOptionsSchema>;
export declare const requestOptionsSchema: z.ZodObject<{
    timeout: z.ZodOptional<z.ZodNumber>;
    verifySsl: z.ZodOptional<z.ZodBoolean>;
    caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
    proxy: z.ZodOptional<z.ZodString>;
    headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
    extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodAny>>;
    noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
    clientCertificate: z.ZodOptional<z.ZodObject<{
        cert: z.ZodString;
        key: z.ZodString;
        passphrase: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>>;
}, z.core.$strip>;
export type RequestOptions = z.infer<typeof requestOptionsSchema>;
export declare const modelRolesSchema: z.ZodEnum<{
    chat: "chat";
    autocomplete: "autocomplete";
    edit: "edit";
    apply: "apply";
    summarize: "summarize";
    viewRead: "viewRead";
    realTimeSearch: "realTimeSearch";
}>;
export type ModelRole = z.infer<typeof modelRolesSchema>;
export declare const modelCapabilitySchema: z.ZodUnion<readonly [z.ZodLiteral<"tool_use">, z.ZodLiteral<"image_input">]>;
export type ModelCapability = z.infer<typeof modelCapabilitySchema>;
export declare const completionOptionsSchema: z.ZodObject<{
    contextLength: z.ZodOptional<z.ZodNumber>;
    maxTokens: z.ZodOptional<z.ZodNumber>;
    temperature: z.ZodOptional<z.ZodNumber>;
    topP: z.ZodOptional<z.ZodNumber>;
    topK: z.ZodOptional<z.ZodNumber>;
    stop: z.ZodOptional<z.ZodArray<z.ZodString>>;
    n: z.ZodOptional<z.ZodNumber>;
}, z.core.$strip>;
export type CompletionOptions = z.infer<typeof completionOptionsSchema>;
declare const promptTemplatesSchema: z.ZodObject<{
    apply: z.ZodOptional<z.ZodString>;
    edit: z.ZodOptional<z.ZodString>;
}, z.core.$strip>;
export type PromptTemplates = z.infer<typeof promptTemplatesSchema>;
export declare const modelSchema: z.ZodObject<{
    provider: z.ZodString;
    name: z.ZodString;
    model: z.ZodString;
    apiKey: z.ZodOptional<z.ZodString>;
    apiBase: z.ZodOptional<z.ZodString>;
    roles: z.ZodOptional<z.ZodArray<z.ZodEnum<{
        chat: "chat";
        autocomplete: "autocomplete";
        edit: "edit";
        apply: "apply";
        summarize: "summarize";
        viewRead: "viewRead";
        realTimeSearch: "realTimeSearch";
    }>>>;
    capabilities: z.ZodOptional<z.ZodArray<z.ZodUnion<readonly [z.ZodLiteral<"tool_use">, z.ZodLiteral<"image_input">]>>>;
    defaultCompletionOptions: z.ZodOptional<z.ZodObject<{
        contextLength: z.ZodOptional<z.ZodNumber>;
        maxTokens: z.ZodOptional<z.ZodNumber>;
        temperature: z.ZodOptional<z.ZodNumber>;
        topP: z.ZodOptional<z.ZodNumber>;
        topK: z.ZodOptional<z.ZodNumber>;
        stop: z.ZodOptional<z.ZodArray<z.ZodString>>;
        n: z.ZodOptional<z.ZodNumber>;
    }, z.core.$strip>>;
    requestOptions: z.ZodOptional<z.ZodObject<{
        timeout: z.ZodOptional<z.ZodNumber>;
        verifySsl: z.ZodOptional<z.ZodBoolean>;
        caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
        proxy: z.ZodOptional<z.ZodString>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodAny>>;
        noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        clientCertificate: z.ZodOptional<z.ZodObject<{
            cert: z.ZodString;
            key: z.ZodString;
            passphrase: z.ZodOptional<z.ZodString>;
        }, z.core.$strip>>;
    }, z.core.$strip>>;
    promptTemplates: z.ZodOptional<z.ZodObject<{
        apply: z.ZodOptional<z.ZodString>;
        edit: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>>;
    env: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnion<readonly [z.ZodString, z.ZodBoolean, z.ZodNumber]>>>;
}, z.core.$strip>;
export declare const partialModelSchema: z.ZodObject<{
    provider: z.ZodOptional<z.ZodString>;
    name: z.ZodOptional<z.ZodString>;
    model: z.ZodOptional<z.ZodString>;
    apiKey: z.ZodOptional<z.ZodOptional<z.ZodString>>;
    apiBase: z.ZodOptional<z.ZodOptional<z.ZodString>>;
    roles: z.ZodOptional<z.ZodOptional<z.ZodArray<z.ZodEnum<{
        chat: "chat";
        autocomplete: "autocomplete";
        edit: "edit";
        apply: "apply";
        summarize: "summarize";
        viewRead: "viewRead";
        realTimeSearch: "realTimeSearch";
    }>>>>;
    capabilities: z.ZodOptional<z.ZodOptional<z.ZodArray<z.ZodUnion<readonly [z.ZodLiteral<"tool_use">, z.ZodLiteral<"image_input">]>>>>;
    defaultCompletionOptions: z.ZodOptional<z.ZodOptional<z.ZodObject<{
        contextLength: z.ZodOptional<z.ZodNumber>;
        maxTokens: z.ZodOptional<z.ZodNumber>;
        temperature: z.ZodOptional<z.ZodNumber>;
        topP: z.ZodOptional<z.ZodNumber>;
        topK: z.ZodOptional<z.ZodNumber>;
        stop: z.ZodOptional<z.ZodArray<z.ZodString>>;
        n: z.ZodOptional<z.ZodNumber>;
    }, z.core.$strip>>>;
    requestOptions: z.ZodOptional<z.ZodOptional<z.ZodObject<{
        timeout: z.ZodOptional<z.ZodNumber>;
        verifySsl: z.ZodOptional<z.ZodBoolean>;
        caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
        proxy: z.ZodOptional<z.ZodString>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodAny>>;
        noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        clientCertificate: z.ZodOptional<z.ZodObject<{
            cert: z.ZodString;
            key: z.ZodString;
            passphrase: z.ZodOptional<z.ZodString>;
        }, z.core.$strip>>;
    }, z.core.$strip>>>;
    promptTemplates: z.ZodOptional<z.ZodOptional<z.ZodObject<{
        apply: z.ZodOptional<z.ZodString>;
        edit: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>>>;
    env: z.ZodOptional<z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnion<readonly [z.ZodString, z.ZodBoolean, z.ZodNumber]>>>>;
}, z.core.$strip>;
export type ModelConfig = z.infer<typeof modelSchema>;
export {};
