import * as z from "zod";
export declare const contextSchema: z.ZodObject<{
    provider: z.ZodString;
    params: z.ZodOptional<z.ZodAny>;
}, z.core.$strip>;
export declare const blockItemWrapperSchema: <T extends z.ZodObject<any>>(schema: T) => z.ZodObject<{
    uses: z.ZodString;
    with: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
    override: z.ZodOptional<z.ZodObject<{
        [x: string]: z.ZodOptional<any>;
    }, z.core.$strip>>;
}, z.core.$strip>;
export declare const blockOrSchema: <T extends z.ZodObject<any>>(schema: T) => z.ZodUnion<readonly [T, z.ZodObject<{
    uses: z.ZodString;
    with: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
    override: z.ZodOptional<z.ZodObject<{
        [x: string]: z.ZodOptional<any>;
    }, z.core.$strip>>;
}, z.core.$strip>]>;
export declare const baseConfigYamlSchema: z.ZodObject<{
    name: z.ZodString;
    version: z.ZodString;
    schema: z.ZodOptional<z.ZodString>;
}, z.core.$strip>;
export declare const configYamlSchema: z.ZodObject<{
    name: z.ZodString;
    version: z.ZodString;
    schema: z.ZodOptional<z.ZodString>;
    models: z.ZodOptional<z.ZodArray<z.ZodUnion<readonly [z.ZodObject<{
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
    }, z.core.$strip>, z.ZodObject<{
        uses: z.ZodString;
        with: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        override: z.ZodOptional<z.ZodObject<{
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
        }, z.core.$strip>>;
    }, z.core.$strip>]>>>;
    context: z.ZodOptional<z.ZodArray<z.ZodUnion<readonly [z.ZodObject<{
        provider: z.ZodString;
        params: z.ZodOptional<z.ZodAny>;
    }, z.core.$strip>, z.ZodObject<{
        uses: z.ZodString;
        with: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        override: z.ZodOptional<z.ZodObject<{
            [x: string]: z.ZodOptional<any>;
        }, z.core.$strip>>;
    }, z.core.$strip>]>>>;
    data: z.ZodOptional<z.ZodArray<z.ZodUnion<readonly [z.ZodObject<{
        name: z.ZodString;
        destination: z.ZodString;
        schema: z.ZodString;
        level: z.ZodOptional<z.ZodUnion<readonly [z.ZodLiteral<"all">, z.ZodLiteral<"noCode">]>>;
        events: z.ZodOptional<z.ZodArray<z.ZodString>>;
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
        apiKey: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>, z.ZodObject<{
        uses: z.ZodString;
        with: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        override: z.ZodOptional<z.ZodObject<{
            [x: string]: z.ZodOptional<any>;
        }, z.core.$strip>>;
    }, z.core.$strip>]>>>;
    mcpServers: z.ZodOptional<z.ZodArray<z.ZodUnion<readonly [z.ZodObject<{
        name: z.ZodString;
        command: z.ZodString;
        faviconUrl: z.ZodOptional<z.ZodString>;
        args: z.ZodOptional<z.ZodArray<z.ZodString>>;
        env: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
    }, z.core.$strip>, z.ZodObject<{
        uses: z.ZodString;
        with: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        override: z.ZodOptional<z.ZodObject<{
            [x: string]: z.ZodOptional<any>;
        }, z.core.$strip>>;
    }, z.core.$strip>]>>>;
    rules: z.ZodOptional<z.ZodArray<z.ZodUnion<readonly [z.ZodString, z.ZodObject<{
        uses: z.ZodString;
        with: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
    }, z.core.$strip>]>>>;
    prompts: z.ZodOptional<z.ZodArray<z.ZodUnion<readonly [z.ZodObject<{
        name: z.ZodString;
        description: z.ZodOptional<z.ZodString>;
        prompt: z.ZodString;
    }, z.core.$strip>, z.ZodObject<{
        uses: z.ZodString;
        with: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        override: z.ZodOptional<z.ZodObject<{
            [x: string]: z.ZodOptional<any>;
        }, z.core.$strip>>;
    }, z.core.$strip>]>>>;
}, z.core.$strip>;
export type ConfigYaml = z.infer<typeof configYamlSchema>;
export declare const assistantUnrolledSchema: z.ZodObject<{
    name: z.ZodString;
    version: z.ZodString;
    schema: z.ZodOptional<z.ZodString>;
    models: z.ZodOptional<z.ZodArray<z.ZodObject<{
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
    }, z.core.$strip>>>;
    context: z.ZodOptional<z.ZodArray<z.ZodObject<{
        provider: z.ZodString;
        params: z.ZodOptional<z.ZodAny>;
    }, z.core.$strip>>>;
    data: z.ZodOptional<z.ZodArray<z.ZodObject<{
        name: z.ZodString;
        destination: z.ZodString;
        schema: z.ZodString;
        level: z.ZodOptional<z.ZodUnion<readonly [z.ZodLiteral<"all">, z.ZodLiteral<"noCode">]>>;
        events: z.ZodOptional<z.ZodArray<z.ZodString>>;
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
        apiKey: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>>>;
    mcpServers: z.ZodOptional<z.ZodArray<z.ZodObject<{
        name: z.ZodString;
        command: z.ZodString;
        faviconUrl: z.ZodOptional<z.ZodString>;
        args: z.ZodOptional<z.ZodArray<z.ZodString>>;
        env: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
    }, z.core.$strip>>>;
    rules: z.ZodOptional<z.ZodArray<z.ZodString>>;
    prompts: z.ZodOptional<z.ZodArray<z.ZodObject<{
        name: z.ZodString;
        description: z.ZodOptional<z.ZodString>;
        prompt: z.ZodString;
    }, z.core.$strip>>>;
}, z.core.$strip>;
export type AssistantUnrolled = z.infer<typeof assistantUnrolledSchema>;
export declare const blockSchema: z.ZodIntersection<z.ZodObject<{
    name: z.ZodString;
    version: z.ZodString;
    schema: z.ZodOptional<z.ZodString>;
}, z.core.$strip>, z.ZodUnion<readonly [z.ZodObject<{
    models: z.ZodArray<z.ZodObject<{
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
    }, z.core.$strip>>;
}, z.core.$strip>, z.ZodObject<{
    context: z.ZodArray<z.ZodObject<{
        provider: z.ZodString;
        params: z.ZodOptional<z.ZodAny>;
    }, z.core.$strip>>;
}, z.core.$strip>, z.ZodObject<{
    data: z.ZodArray<z.ZodObject<{
        name: z.ZodString;
        destination: z.ZodString;
        schema: z.ZodString;
        level: z.ZodOptional<z.ZodUnion<readonly [z.ZodLiteral<"all">, z.ZodLiteral<"noCode">]>>;
        events: z.ZodOptional<z.ZodArray<z.ZodString>>;
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
        apiKey: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>>;
}, z.core.$strip>, z.ZodObject<{
    mcpServers: z.ZodArray<z.ZodObject<{
        name: z.ZodString;
        command: z.ZodString;
        faviconUrl: z.ZodOptional<z.ZodString>;
        args: z.ZodOptional<z.ZodArray<z.ZodString>>;
        env: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
    }, z.core.$strip>>;
}, z.core.$strip>, z.ZodObject<{
    rules: z.ZodArray<z.ZodString>;
}, z.core.$strip>, z.ZodObject<{
    prompts: z.ZodArray<z.ZodObject<{
        name: z.ZodString;
        description: z.ZodOptional<z.ZodString>;
        prompt: z.ZodString;
    }, z.core.$strip>>;
}, z.core.$strip>]>>;
export type Block = z.infer<typeof blockSchema>;
