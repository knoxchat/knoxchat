import * as z from "zod";
export declare const ClientCertificateOptionsSchema: z.ZodObject<{
    cert: z.ZodString;
    key: z.ZodString;
    passphrase: z.ZodOptional<z.ZodString>;
}, z.core.$strip>;
export declare const RequestOptionsSchema: z.ZodObject<{
    timeout: z.ZodOptional<z.ZodNumber>;
    verifySsl: z.ZodOptional<z.ZodBoolean>;
    caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
    proxy: z.ZodOptional<z.ZodString>;
    headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
    extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
    noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
    clientCertificate: z.ZodOptional<z.ZodLazy<z.ZodObject<{
        cert: z.ZodString;
        key: z.ZodString;
        passphrase: z.ZodOptional<z.ZodString>;
    }, z.core.$strip>>>;
}, z.core.$strip>;
export declare const BaseConfig: z.ZodObject<{
    provider: z.ZodString;
    requestOptions: z.ZodOptional<z.ZodObject<{
        timeout: z.ZodOptional<z.ZodNumber>;
        verifySsl: z.ZodOptional<z.ZodBoolean>;
        caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
        proxy: z.ZodOptional<z.ZodString>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
        noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        clientCertificate: z.ZodOptional<z.ZodLazy<z.ZodObject<{
            cert: z.ZodString;
            key: z.ZodString;
            passphrase: z.ZodOptional<z.ZodString>;
        }, z.core.$strip>>>;
    }, z.core.$strip>>;
}, z.core.$strip>;
export declare const BasePlusConfig: z.ZodObject<{
    provider: z.ZodString;
    requestOptions: z.ZodOptional<z.ZodObject<{
        timeout: z.ZodOptional<z.ZodNumber>;
        verifySsl: z.ZodOptional<z.ZodBoolean>;
        caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
        proxy: z.ZodOptional<z.ZodString>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
        noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        clientCertificate: z.ZodOptional<z.ZodLazy<z.ZodObject<{
            cert: z.ZodString;
            key: z.ZodString;
            passphrase: z.ZodOptional<z.ZodString>;
        }, z.core.$strip>>>;
    }, z.core.$strip>>;
    apiBase: z.ZodOptional<z.ZodString>;
    apiKey: z.ZodOptional<z.ZodString>;
}, z.core.$strip>;
export declare const OpenAIConfigSchema: z.ZodObject<{
    requestOptions: z.ZodOptional<z.ZodObject<{
        timeout: z.ZodOptional<z.ZodNumber>;
        verifySsl: z.ZodOptional<z.ZodBoolean>;
        caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
        proxy: z.ZodOptional<z.ZodString>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
        noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        clientCertificate: z.ZodOptional<z.ZodLazy<z.ZodObject<{
            cert: z.ZodString;
            key: z.ZodString;
            passphrase: z.ZodOptional<z.ZodString>;
        }, z.core.$strip>>>;
    }, z.core.$strip>>;
    apiBase: z.ZodOptional<z.ZodString>;
    apiKey: z.ZodOptional<z.ZodString>;
    provider: z.ZodUnion<readonly [z.ZodLiteral<"openai">, z.ZodLiteral<"knoxchat">]>;
}, z.core.$strip>;
export type OpenAIConfig = z.infer<typeof OpenAIConfigSchema>;
export declare const MockConfigSchema: z.ZodObject<{
    requestOptions: z.ZodOptional<z.ZodObject<{
        timeout: z.ZodOptional<z.ZodNumber>;
        verifySsl: z.ZodOptional<z.ZodBoolean>;
        caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
        proxy: z.ZodOptional<z.ZodString>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
        noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        clientCertificate: z.ZodOptional<z.ZodLazy<z.ZodObject<{
            cert: z.ZodString;
            key: z.ZodString;
            passphrase: z.ZodOptional<z.ZodString>;
        }, z.core.$strip>>>;
    }, z.core.$strip>>;
    apiBase: z.ZodOptional<z.ZodString>;
    apiKey: z.ZodOptional<z.ZodString>;
    provider: z.ZodLiteral<"mock">;
}, z.core.$strip>;
export type MockConfig = z.infer<typeof MockConfigSchema>;
export declare const AnthropicConfigSchema: z.ZodObject<{
    requestOptions: z.ZodOptional<z.ZodObject<{
        timeout: z.ZodOptional<z.ZodNumber>;
        verifySsl: z.ZodOptional<z.ZodBoolean>;
        caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
        proxy: z.ZodOptional<z.ZodString>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
        noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        clientCertificate: z.ZodOptional<z.ZodLazy<z.ZodObject<{
            cert: z.ZodString;
            key: z.ZodString;
            passphrase: z.ZodOptional<z.ZodString>;
        }, z.core.$strip>>>;
    }, z.core.$strip>>;
    apiBase: z.ZodOptional<z.ZodString>;
    provider: z.ZodLiteral<"anthropic">;
    apiKey: z.ZodString;
}, z.core.$strip>;
export type AnthropicConfig = z.infer<typeof AnthropicConfigSchema>;
export declare const LLMConfigSchema: z.ZodDiscriminatedUnion<[z.ZodObject<{
    requestOptions: z.ZodOptional<z.ZodObject<{
        timeout: z.ZodOptional<z.ZodNumber>;
        verifySsl: z.ZodOptional<z.ZodBoolean>;
        caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
        proxy: z.ZodOptional<z.ZodString>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
        noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        clientCertificate: z.ZodOptional<z.ZodLazy<z.ZodObject<{
            cert: z.ZodString;
            key: z.ZodString;
            passphrase: z.ZodOptional<z.ZodString>;
        }, z.core.$strip>>>;
    }, z.core.$strip>>;
    apiBase: z.ZodOptional<z.ZodString>;
    apiKey: z.ZodOptional<z.ZodString>;
    provider: z.ZodUnion<readonly [z.ZodLiteral<"openai">, z.ZodLiteral<"knoxchat">]>;
}, z.core.$strip>, z.ZodObject<{
    requestOptions: z.ZodOptional<z.ZodObject<{
        timeout: z.ZodOptional<z.ZodNumber>;
        verifySsl: z.ZodOptional<z.ZodBoolean>;
        caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
        proxy: z.ZodOptional<z.ZodString>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
        noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        clientCertificate: z.ZodOptional<z.ZodLazy<z.ZodObject<{
            cert: z.ZodString;
            key: z.ZodString;
            passphrase: z.ZodOptional<z.ZodString>;
        }, z.core.$strip>>>;
    }, z.core.$strip>>;
    apiBase: z.ZodOptional<z.ZodString>;
    provider: z.ZodLiteral<"anthropic">;
    apiKey: z.ZodString;
}, z.core.$strip>, z.ZodObject<{
    requestOptions: z.ZodOptional<z.ZodObject<{
        timeout: z.ZodOptional<z.ZodNumber>;
        verifySsl: z.ZodOptional<z.ZodBoolean>;
        caBundlePath: z.ZodOptional<z.ZodUnion<readonly [z.ZodString, z.ZodArray<z.ZodString>]>>;
        proxy: z.ZodOptional<z.ZodString>;
        headers: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        extraBodyProperties: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
        noProxy: z.ZodOptional<z.ZodArray<z.ZodString>>;
        clientCertificate: z.ZodOptional<z.ZodLazy<z.ZodObject<{
            cert: z.ZodString;
            key: z.ZodString;
            passphrase: z.ZodOptional<z.ZodString>;
        }, z.core.$strip>>>;
    }, z.core.$strip>>;
    apiBase: z.ZodOptional<z.ZodString>;
    apiKey: z.ZodOptional<z.ZodString>;
    provider: z.ZodLiteral<"mock">;
}, z.core.$strip>], "provider">;
export type LLMConfig = z.infer<typeof LLMConfigSchema>;
