import { z } from "zod";
export declare const completionOptionsSchema: z.ZodObject<{
    temperature: z.ZodOptional<z.ZodNumber>;
    topP: z.ZodOptional<z.ZodNumber>;
    topK: z.ZodOptional<z.ZodNumber>;
    minP: z.ZodOptional<z.ZodNumber>;
    presencePenalty: z.ZodOptional<z.ZodNumber>;
    frequencyPenalty: z.ZodOptional<z.ZodNumber>;
    mirostat: z.ZodOptional<z.ZodNumber>;
    stop: z.ZodOptional<z.ZodArray<z.ZodString>>;
    maxTokens: z.ZodOptional<z.ZodNumber>;
    numThreads: z.ZodOptional<z.ZodNumber>;
    useMmap: z.ZodOptional<z.ZodBoolean>;
    keepAlive: z.ZodOptional<z.ZodNumber>;
    raw: z.ZodOptional<z.ZodBoolean>;
    stream: z.ZodOptional<z.ZodBoolean>;
}, z.core.$strip>;
export type CompletionOptions = z.infer<typeof completionOptionsSchema>;
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
export declare const modelDescriptionSchema: z.ZodObject<{
    title: z.ZodString;
    provider: z.ZodEnum<{
        openai: "openai";
        anthropic: "anthropic";
    }>;
    model: z.ZodString;
    apiKey: z.ZodOptional<z.ZodString>;
    apiBase: z.ZodOptional<z.ZodString>;
    contextLength: z.ZodOptional<z.ZodNumber>;
    template: z.ZodOptional<z.ZodEnum<{
        anthropic: "anthropic";
        none: "none";
    }>>;
    completionOptions: z.ZodOptional<z.ZodObject<{
        temperature: z.ZodOptional<z.ZodNumber>;
        topP: z.ZodOptional<z.ZodNumber>;
        topK: z.ZodOptional<z.ZodNumber>;
        minP: z.ZodOptional<z.ZodNumber>;
        presencePenalty: z.ZodOptional<z.ZodNumber>;
        frequencyPenalty: z.ZodOptional<z.ZodNumber>;
        mirostat: z.ZodOptional<z.ZodNumber>;
        stop: z.ZodOptional<z.ZodArray<z.ZodString>>;
        maxTokens: z.ZodOptional<z.ZodNumber>;
        numThreads: z.ZodOptional<z.ZodNumber>;
        useMmap: z.ZodOptional<z.ZodBoolean>;
        keepAlive: z.ZodOptional<z.ZodNumber>;
        raw: z.ZodOptional<z.ZodBoolean>;
        stream: z.ZodOptional<z.ZodBoolean>;
    }, z.core.$strip>>;
    systemMessage: z.ZodOptional<z.ZodString>;
    requestOptions: z.ZodOptional<z.ZodObject<{
        timeout: z.ZodOptional<z.ZodNumber>;
        verifySsl: z.ZodOptional<z.ZodBoolean>;
        caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
        proxy: z.ZodOptional<z.ZodString>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodAny>>;
        noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
    }, z.core.$strip>>;
    promptTemplates: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
}, z.core.$strip>;
export type ModelDescription = z.infer<typeof modelDescriptionSchema>;
export declare const uiOptionsSchema: z.ZodObject<{
    codeBlockToolbarPosition: z.ZodOptional<z.ZodEnum<{
        top: "top";
        bottom: "bottom";
    }>>;
    fontSize: z.ZodOptional<z.ZodNumber>;
    displayRawMarkdown: z.ZodOptional<z.ZodBoolean>;
    showChatScrollbar: z.ZodOptional<z.ZodBoolean>;
    codeWrap: z.ZodOptional<z.ZodBoolean>;
}, z.core.$strip>;
export type UiOptions = z.infer<typeof uiOptionsSchema>;
export declare const tabAutocompleteOptionsSchema: z.ZodObject<{
    disable: z.ZodBoolean;
    maxPromptTokens: z.ZodNumber;
    debounceDelay: z.ZodNumber;
    maxSuffixPercentage: z.ZodNumber;
    prefixPercentage: z.ZodNumber;
    transform: z.ZodOptional<z.ZodBoolean>;
    template: z.ZodOptional<z.ZodString>;
    multilineCompletions: z.ZodEnum<{
        never: "never";
        always: "always";
        auto: "auto";
    }>;
    slidingWindowPrefixPercentage: z.ZodNumber;
    slidingWindowSize: z.ZodNumber;
    useCache: z.ZodBoolean;
    onlyMyCode: z.ZodBoolean;
    useRecentlyEdited: z.ZodBoolean;
    disableInFiles: z.ZodOptional<z.ZodArray<z.ZodString>>;
    useImports: z.ZodOptional<z.ZodBoolean>;
}, z.core.$strip>;
export type TabAutocompleteOptions = z.infer<typeof tabAutocompleteOptionsSchema>;
export declare const slashCommandSchema: z.ZodObject<{
    name: z.ZodString;
    description: z.ZodString;
    params: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodAny>>;
}, z.core.$strip>;
export type SlashCommand = z.infer<typeof slashCommandSchema>;
export declare const customCommandSchema: z.ZodObject<{
    name: z.ZodString;
    description: z.ZodString;
    params: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodAny>>;
}, z.core.$strip>;
export type CustomCommand = z.infer<typeof customCommandSchema>;
export declare const contextProviderSchema: z.ZodObject<{
    name: z.ZodString;
    params: z.ZodRecord<z.ZodString, z.ZodAny>;
}, z.core.$strip>;
export type ContextProvider = z.infer<typeof contextProviderSchema>;
export declare const analyticsSchema: z.ZodObject<{
    provider: z.ZodEnum<{
        amplitude: "amplitude";
        segment: "segment";
        logstash: "logstash";
        mixpanel: "mixpanel";
        splunk: "splunk";
        datadog: "datadog";
    }>;
    url: z.ZodOptional<z.ZodString>;
    clientKey: z.ZodOptional<z.ZodString>;
}, z.core.$strip>;
export type Analytics = z.infer<typeof analyticsSchema>;
export declare const devDataSchema: z.ZodObject<{
    url: z.ZodOptional<z.ZodString>;
}, z.core.$strip>;
export type DevData = z.infer<typeof devDataSchema>;
export declare const controlPlaneConfigSchema: z.ZodObject<{
    useKnoxForTeamsProxy: z.ZodOptional<z.ZodBoolean>;
    proxyUrl: z.ZodOptional<z.ZodString>;
}, z.core.$strip>;
export declare const configJsonSchema: z.ZodObject<{
    models: z.ZodArray<z.ZodObject<{
        title: z.ZodString;
        provider: z.ZodEnum<{
            openai: "openai";
            anthropic: "anthropic";
        }>;
        model: z.ZodString;
        apiKey: z.ZodOptional<z.ZodString>;
        apiBase: z.ZodOptional<z.ZodString>;
        contextLength: z.ZodOptional<z.ZodNumber>;
        template: z.ZodOptional<z.ZodEnum<{
            anthropic: "anthropic";
            none: "none";
        }>>;
        completionOptions: z.ZodOptional<z.ZodObject<{
            temperature: z.ZodOptional<z.ZodNumber>;
            topP: z.ZodOptional<z.ZodNumber>;
            topK: z.ZodOptional<z.ZodNumber>;
            minP: z.ZodOptional<z.ZodNumber>;
            presencePenalty: z.ZodOptional<z.ZodNumber>;
            frequencyPenalty: z.ZodOptional<z.ZodNumber>;
            mirostat: z.ZodOptional<z.ZodNumber>;
            stop: z.ZodOptional<z.ZodArray<z.ZodString>>;
            maxTokens: z.ZodOptional<z.ZodNumber>;
            numThreads: z.ZodOptional<z.ZodNumber>;
            useMmap: z.ZodOptional<z.ZodBoolean>;
            keepAlive: z.ZodOptional<z.ZodNumber>;
            raw: z.ZodOptional<z.ZodBoolean>;
            stream: z.ZodOptional<z.ZodBoolean>;
        }, z.core.$strip>>;
        systemMessage: z.ZodOptional<z.ZodString>;
        requestOptions: z.ZodOptional<z.ZodObject<{
            timeout: z.ZodOptional<z.ZodNumber>;
            verifySsl: z.ZodOptional<z.ZodBoolean>;
            caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
            proxy: z.ZodOptional<z.ZodString>;
            headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
            extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodAny>>;
            noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        }, z.core.$strip>>;
        promptTemplates: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
    }, z.core.$strip>>;
    tabAutocompleteModel: z.ZodOptional<z.ZodObject<{
        title: z.ZodString;
        provider: z.ZodEnum<{
            openai: "openai";
            anthropic: "anthropic";
        }>;
        model: z.ZodString;
        apiKey: z.ZodOptional<z.ZodString>;
        apiBase: z.ZodOptional<z.ZodString>;
        contextLength: z.ZodOptional<z.ZodNumber>;
        template: z.ZodOptional<z.ZodEnum<{
            anthropic: "anthropic";
            none: "none";
        }>>;
        completionOptions: z.ZodOptional<z.ZodObject<{
            temperature: z.ZodOptional<z.ZodNumber>;
            topP: z.ZodOptional<z.ZodNumber>;
            topK: z.ZodOptional<z.ZodNumber>;
            minP: z.ZodOptional<z.ZodNumber>;
            presencePenalty: z.ZodOptional<z.ZodNumber>;
            frequencyPenalty: z.ZodOptional<z.ZodNumber>;
            mirostat: z.ZodOptional<z.ZodNumber>;
            stop: z.ZodOptional<z.ZodArray<z.ZodString>>;
            maxTokens: z.ZodOptional<z.ZodNumber>;
            numThreads: z.ZodOptional<z.ZodNumber>;
            useMmap: z.ZodOptional<z.ZodBoolean>;
            keepAlive: z.ZodOptional<z.ZodNumber>;
            raw: z.ZodOptional<z.ZodBoolean>;
            stream: z.ZodOptional<z.ZodBoolean>;
        }, z.core.$strip>>;
        systemMessage: z.ZodOptional<z.ZodString>;
        requestOptions: z.ZodOptional<z.ZodObject<{
            timeout: z.ZodOptional<z.ZodNumber>;
            verifySsl: z.ZodOptional<z.ZodBoolean>;
            caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
            proxy: z.ZodOptional<z.ZodString>;
            headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
            extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodAny>>;
            noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        }, z.core.$strip>>;
        promptTemplates: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
    }, z.core.$strip>>;
    analytics: z.ZodObject<{
        provider: z.ZodEnum<{
            amplitude: "amplitude";
            segment: "segment";
            logstash: "logstash";
            mixpanel: "mixpanel";
            splunk: "splunk";
            datadog: "datadog";
        }>;
        url: z.ZodOptional<z.ZodString>;
        clientKey: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>;
    devData: z.ZodObject<{
        url: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>;
    systemMessage: z.ZodOptional<z.ZodString>;
    completionOptions: z.ZodOptional<z.ZodObject<{
        temperature: z.ZodOptional<z.ZodNumber>;
        topP: z.ZodOptional<z.ZodNumber>;
        topK: z.ZodOptional<z.ZodNumber>;
        minP: z.ZodOptional<z.ZodNumber>;
        presencePenalty: z.ZodOptional<z.ZodNumber>;
        frequencyPenalty: z.ZodOptional<z.ZodNumber>;
        mirostat: z.ZodOptional<z.ZodNumber>;
        stop: z.ZodOptional<z.ZodArray<z.ZodString>>;
        maxTokens: z.ZodOptional<z.ZodNumber>;
        numThreads: z.ZodOptional<z.ZodNumber>;
        useMmap: z.ZodOptional<z.ZodBoolean>;
        keepAlive: z.ZodOptional<z.ZodNumber>;
        raw: z.ZodOptional<z.ZodBoolean>;
        stream: z.ZodOptional<z.ZodBoolean>;
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
    slashCommands: z.ZodOptional<z.ZodArray<z.ZodObject<{
        name: z.ZodString;
        description: z.ZodString;
        params: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodAny>>;
    }, z.core.$strip>>>;
    customCommands: z.ZodOptional<z.ZodArray<z.ZodObject<{
        name: z.ZodString;
        description: z.ZodString;
        params: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodAny>>;
    }, z.core.$strip>>>;
    contextProviders: z.ZodOptional<z.ZodArray<z.ZodObject<{
        name: z.ZodString;
        params: z.ZodRecord<z.ZodString, z.ZodAny>;
    }, z.core.$strip>>>;
    tabAutocompleteOptions: z.ZodOptional<z.ZodObject<{
        disable: z.ZodBoolean;
        maxPromptTokens: z.ZodNumber;
        debounceDelay: z.ZodNumber;
        maxSuffixPercentage: z.ZodNumber;
        prefixPercentage: z.ZodNumber;
        transform: z.ZodOptional<z.ZodBoolean>;
        template: z.ZodOptional<z.ZodString>;
        multilineCompletions: z.ZodEnum<{
            never: "never";
            always: "always";
            auto: "auto";
        }>;
        slidingWindowPrefixPercentage: z.ZodNumber;
        slidingWindowSize: z.ZodNumber;
        useCache: z.ZodBoolean;
        onlyMyCode: z.ZodBoolean;
        useRecentlyEdited: z.ZodBoolean;
        disableInFiles: z.ZodOptional<z.ZodArray<z.ZodString>>;
        useImports: z.ZodOptional<z.ZodBoolean>;
    }, z.core.$strip>>;
    ui: z.ZodOptional<z.ZodObject<{
        codeBlockToolbarPosition: z.ZodOptional<z.ZodEnum<{
            top: "top";
            bottom: "bottom";
        }>>;
        fontSize: z.ZodOptional<z.ZodNumber>;
        displayRawMarkdown: z.ZodOptional<z.ZodBoolean>;
        showChatScrollbar: z.ZodOptional<z.ZodBoolean>;
        codeWrap: z.ZodOptional<z.ZodBoolean>;
    }, z.core.$strip>>;
    controlPlane: z.ZodOptional<z.ZodObject<{
        useKnoxForTeamsProxy: z.ZodOptional<z.ZodBoolean>;
        proxyUrl: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>>;
}, z.core.$strip>;
export type ConfigJson = z.infer<typeof configJsonSchema>;
