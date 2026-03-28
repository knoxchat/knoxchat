import { z } from "zod";
export const completionOptionsSchema = z.object({
    temperature: z.number().optional(),
    topP: z.number().optional(),
    topK: z.number().optional(),
    minP: z.number().optional(),
    presencePenalty: z.number().optional(),
    frequencyPenalty: z.number().optional(),
    mirostat: z.number().optional(),
    stop: z.array(z.string()).optional(),
    maxTokens: z.number().optional(),
    numThreads: z.number().optional(),
    useMmap: z.boolean().optional(),
    keepAlive: z.number().optional(),
    raw: z.boolean().optional(),
    stream: z.boolean().optional(),
});
export const clientCertificateOptionsSchema = z.object({
    cert: z.string(),
    key: z.string(),
    passphrase: z.string().optional(),
});
export const requestOptionsSchema = z.object({
    timeout: z.number().optional(),
    verifySsl: z.boolean().optional(),
    caBundlePath: z.union([z.string(), z.array(z.string())]).optional(),
    proxy: z.string().optional(),
    headers: z.record(z.string(), z.string()).optional(),
    extraBodyProperties: z.record(z.string(), z.any()).optional(),
    noProxy: z.array(z.string()).optional(),
    clientCertificate: clientCertificateOptionsSchema.optional(),
});
export const modelDescriptionSchema = z.object({
    title: z.string(),
    provider: z.enum([
        "openai",
        "anthropic"
    ]),
    model: z.string(),
    apiKey: z.string().optional(),
    apiBase: z.string().optional(),
    contextLength: z.number().optional(),
    template: z
        .enum([
        "anthropic",
        "none",
    ])
        .optional(),
    completionOptions: completionOptionsSchema.optional(),
    systemMessage: z.string().optional(),
    requestOptions: z
        .object({
        timeout: z.number().optional(),
        verifySsl: z.boolean().optional(),
        caBundlePath: z.union([z.string(), z.array(z.string())]).optional(),
        proxy: z.string().optional(),
        headers: z.record(z.string(), z.string()).optional(),
        extraBodyProperties: z.record(z.string(), z.any()).optional(),
        noProxy: z.array(z.string()).optional(),
    })
        .optional(),
    promptTemplates: z.record(z.string(), z.string()).optional(),
});
export const uiOptionsSchema = z.object({
    codeBlockToolbarPosition: z.enum(["top", "bottom"]).optional(),
    fontSize: z.number().optional(),
    displayRawMarkdown: z.boolean().optional(),
    showChatScrollbar: z.boolean().optional(),
    codeWrap: z.boolean().optional(),
});
export const tabAutocompleteOptionsSchema = z.object({
    disable: z.boolean(),
    maxPromptTokens: z.number(),
    debounceDelay: z.number(),
    maxSuffixPercentage: z.number(),
    prefixPercentage: z.number(),
    transform: z.boolean().optional(),
    template: z.string().optional(),
    multilineCompletions: z.enum(["always", "never", "auto"]),
    slidingWindowPrefixPercentage: z.number(),
    slidingWindowSize: z.number(),
    useCache: z.boolean(),
    onlyMyCode: z.boolean(),
    useRecentlyEdited: z.boolean(),
    disableInFiles: z.array(z.string()).optional(),
    useImports: z.boolean().optional(),
});
export const slashCommandSchema = z.object({
    name: z.string(),
    description: z.string(),
    params: z.record(z.string(), z.any()).optional(),
});
export const customCommandSchema = z.object({
    name: z.string(),
    description: z.string(),
    params: z.record(z.string(), z.any()).optional(),
});
export const contextProviderSchema = z.object({
    name: z.string(),
    params: z.record(z.string(), z.any()),
});
export const analyticsSchema = z.object({
    provider: z.enum([
        "amplitude",
        "segment",
        "logstash",
        "mixpanel",
        "splunk",
        "datadog",
    ]),
    url: z.string().optional(),
    clientKey: z.string().optional(),
});
export const devDataSchema = z.object({
    url: z.string().optional(),
});
export const controlPlaneConfigSchema = z.object({
    useKnoxForTeamsProxy: z.boolean().optional(),
    proxyUrl: z.string().optional(),
});
export const configJsonSchema = z.object({
    models: z.array(modelDescriptionSchema),
    tabAutocompleteModel: modelDescriptionSchema.optional(),
    analytics: analyticsSchema,
    devData: devDataSchema,
    allowAnonymousTelemetry: z.boolean().optional(),
    systemMessage: z.string().optional(),
    completionOptions: completionOptionsSchema.optional(),
    requestOptions: requestOptionsSchema.optional(),
    slashCommands: z.array(slashCommandSchema).optional(),
    customCommands: z.array(customCommandSchema).optional(),
    contextProviders: z.array(contextProviderSchema).optional(),
    tabAutocompleteOptions: tabAutocompleteOptionsSchema.optional(),
    ui: uiOptionsSchema.optional(),
    controlPlane: controlPlaneConfigSchema.optional(),
});
