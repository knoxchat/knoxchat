import * as z from "zod";
import { dataSchema } from "./data/index.js";
import { modelSchema, partialModelSchema } from "./models.js";
export const contextSchema = z.object({
    provider: z.string(),
    params: z.any().optional(),
});
const promptSchema = z.object({
    name: z.string(),
    description: z.string().optional(),
    prompt: z.string(),
});
export const blockItemWrapperSchema = (schema) => z.object({
    uses: z.string(),
    with: z.record(z.string(), z.string()).optional(),
    override: schema.partial().optional(),
});
export const blockOrSchema = (schema) => z.union([schema, blockItemWrapperSchema(schema)]);
export const baseConfigYamlSchema = z.object({
    name: z.string(),
    version: z.string(),
    schema: z.string().optional(),
});
export const configYamlSchema = baseConfigYamlSchema.extend({
    models: z
        .array(z.union([
        modelSchema,
        z.object({
            uses: z.string(),
            with: z.record(z.string(), z.string()).optional(),
            override: partialModelSchema.optional(),
        }),
    ]))
        .optional(),
    context: z.array(blockOrSchema(contextSchema)).optional(),
    data: z.array(blockOrSchema(dataSchema)).optional(),
    rules: z
        .array(z.union([
        z.string(),
        z.object({
            uses: z.string(),
            with: z.record(z.string(), z.string()).optional(),
        }),
    ]))
        .optional(),
    prompts: z.array(blockOrSchema(promptSchema)).optional(),
});
export const assistantUnrolledSchema = baseConfigYamlSchema.extend({
    models: z.array(modelSchema).optional(),
    context: z.array(contextSchema).optional(),
    data: z.array(dataSchema).optional(),
    rules: z.array(z.string()).optional(),
    prompts: z.array(promptSchema).optional(),
});
export const blockSchema = baseConfigYamlSchema.and(z.union([
    z.object({ models: z.array(modelSchema).length(1) }),
    z.object({ context: z.array(contextSchema).length(1) }),
    z.object({ data: z.array(dataSchema).length(1) }),
    z.object({ rules: z.array(z.string()).length(1) }),
    z.object({ prompts: z.array(promptSchema).length(1) }),
]));
