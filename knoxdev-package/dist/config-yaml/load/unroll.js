import * as YAML from "yaml";
import { encodeSecretLocation } from "../interfaces/SecretResult.js";
import { decodeFQSN, decodeFullSlug, encodeFQSN, encodePackageSlug, } from "../interfaces/slugs.js";
import { assistantUnrolledSchema, blockSchema, configYamlSchema, } from "../schemas/index.js";
import { useProxyForUnrenderedSecrets } from "./clientRender.js";
export function parseConfigYaml(configYaml) {
    try {
        const parsed = YAML.parse(configYaml);
        const result = configYamlSchema.safeParse(parsed);
        if (result.success) {
            return result.data;
        }
        throw new Error(result.error.issues
            .map((e) => `${e.path.join(".")}: ${e.message}`)
            .join(""));
    }
    catch (e) {
        console.log("Failed to parse rolled assistant:", configYaml);
        throw new Error(`Failed to parse assistant:\n${e instanceof Error ? e.message : e}`);
    }
}
export function parseAssistantUnrolled(configYaml) {
    try {
        const parsed = YAML.parse(configYaml);
        const result = assistantUnrolledSchema.parse(parsed);
        return result;
    }
    catch (e) {
        throw new Error(`Failed to parse unrolled assistant: ${e.message}\n\n${configYaml}`);
    }
}
export function parseBlock(configYaml) {
    try {
        const parsed = YAML.parse(configYaml);
        const result = blockSchema.parse(parsed);
        return result;
    }
    catch (e) {
        throw new Error(`Failed to parse block: ${e.message}`);
    }
}
const TEMPLATE_VAR_REGEX = /\${{[\s]*([^}\s]+)[\s]*}}/g;
export function getTemplateVariables(templatedYaml) {
    const variables = new Set();
    const matches = templatedYaml.matchAll(TEMPLATE_VAR_REGEX);
    for (const match of matches) {
        variables.add(match[1]);
    }
    return Array.from(variables);
}
export function fillTemplateVariables(templatedYaml, data) {
    return templatedYaml.replace(TEMPLATE_VAR_REGEX, (match, variableName) => {
        // Inject data
        if (variableName in data) {
            return data[variableName];
        }
        // If variable doesn't exist, return the original expression
        return match;
    });
}
function flattenTemplateData(templateData) {
    const flattened = {};
    if (templateData.inputs) {
        for (const [key, value] of Object.entries(templateData.inputs)) {
            flattened[`inputs.${key}`] = value;
        }
    }
    if (templateData.secrets) {
        for (const [key, value] of Object.entries(templateData.secrets)) {
            flattened[`secrets.${key}`] = value;
        }
    }
    return flattened;
}
function secretToFQSNMap(secretNames, parentPackages) {
    const map = {};
    for (const secret of secretNames) {
        const parentSlugs = parentPackages.map(encodePackageSlug);
        const parts = [...parentSlugs, secret];
        const fqsn = parts.join("/");
        map[secret] = `\${{ secrets.${fqsn} }}`;
    }
    return map;
}
function extractFQSNMap(rawContent, parentPackages) {
    const templateVars = getTemplateVariables(rawContent);
    const secrets = templateVars
        .filter((v) => v.startsWith("secrets."))
        .map((v) => v.replace("secrets.", ""));
    return secretToFQSNMap(secrets, parentPackages);
}
/**
 * All template vars are already FQSNs, here we just resolve them to either locations or values
 */
async function extractRenderedSecretsMap(rawContent, platformClient) {
    // Get all template variables
    const templateVars = getTemplateVariables(rawContent);
    const secrets = templateVars
        .filter((v) => v.startsWith("secrets."))
        .map((v) => v.replace("secrets.", ""));
    const fqsns = secrets.map(decodeFQSN);
    // FQSN -> SecretResult
    const secretResults = await platformClient.resolveFQSNs(fqsns);
    const map = {};
    for (const secretResult of secretResults) {
        if (!secretResult) {
            continue;
        }
        // User secrets are rendered
        if ("value" in secretResult) {
            map[encodeFQSN(secretResult.fqsn)] = secretResult.value;
        }
        else {
            // Other secrets are rendered as secret locations and then converted to proxy types later
            map[encodeFQSN(secretResult.fqsn)] =
                `\${{ secrets.${encodeSecretLocation(secretResult.secretLocation)} }}`;
        }
    }
    return map;
}
export async function unrollAssistant(fullSlug, registry, options) {
    const assistantSlug = decodeFullSlug(fullSlug);
    // Request the content from the registry
    const rawContent = await registry.getContent(assistantSlug);
    return unrollAssistantFromContent(assistantSlug, rawContent, registry, options);
}
function renderTemplateData(rawYaml, templateData) {
    const fullTemplateData = {
        inputs: {},
        secrets: {},
        knox: {},
        ...templateData,
    };
    const templatedYaml = fillTemplateVariables(rawYaml, flattenTemplateData(fullTemplateData));
    return templatedYaml;
}
export async function unrollAssistantFromContent(assistantSlug, rawYaml, registry, options) {
    // Parse string to Zod-validated YAML
    let parsedYaml = parseConfigYaml(rawYaml);
    // Unroll blocks and convert their secrets to FQSNs
    const unrolledAssistant = await unrollBlocks(parsedYaml, registry);
    // Back to a string so we can fill in template variables
    const rawUnrolledYaml = YAML.stringify(unrolledAssistant);
    // Convert all of the template variables to FQSNs
    // Secrets from the block will have the assistant slug prepended to the FQSN
    const templatedYaml = renderTemplateData(rawUnrolledYaml, {
        secrets: extractFQSNMap(rawUnrolledYaml, [assistantSlug]),
    });
    if (!options.renderSecrets) {
        return parseAssistantUnrolled(templatedYaml);
    }
    // Render secret values/locations for client
    const secrets = await extractRenderedSecretsMap(templatedYaml, options.platformClient);
    const renderedYaml = renderTemplateData(templatedYaml, {
        secrets,
    });
    // Parse again and replace models with proxy versions where secrets weren't rendered
    const finalConfig = useProxyForUnrenderedSecrets(parseAssistantUnrolled(renderedYaml), assistantSlug, options.orgScopeId, options.onPremProxyUrl);
    return finalConfig;
}
export async function unrollBlocks(assistant, registry) {
    const unrolledAssistant = {
        name: assistant.name,
        version: assistant.version,
    };
    const sections = ["models", "context", "data", "prompts"];
    // For each section, replace "uses/with" blocks with the real thing
    for (const section of sections) {
        if (assistant[section]) {
            const sectionBlocks = [];
            for (const unrolledBlock of assistant[section]) {
                // "uses/with" block
                if ("uses" in unrolledBlock) {
                    const blockConfigYaml = await resolveBlock(decodeFullSlug(unrolledBlock.uses), unrolledBlock.with, registry);
                    const block = blockConfigYaml[section]?.[0];
                    if (block) {
                        sectionBlocks.push(mergeOverrides(block, unrolledBlock.override ?? {}));
                    }
                }
                else {
                    // Normal block
                    sectionBlocks.push(unrolledBlock);
                }
            }
            unrolledAssistant[section] = sectionBlocks;
        }
    }
    // Rules are a bit different because they're just strings, so handle separately
    if (assistant.rules) {
        const rules = [];
        for (const rule of assistant.rules) {
            if (typeof rule === "string") {
                rules.push(rule);
            }
            else {
                const blockConfigYaml = await resolveBlock(decodeFullSlug(rule.uses), rule.with, registry);
                const block = blockConfigYaml.rules?.[0];
                if (block) {
                    rules.push(block);
                }
            }
        }
        unrolledAssistant.rules = rules;
    }
    return unrolledAssistant;
}
export async function resolveBlock(fullSlug, inputs, registry) {
    // Retrieve block raw yaml
    const rawYaml = await registry.getContent(fullSlug);
    if (rawYaml === undefined) {
        throw new Error(`Block ${fullSlug.ownerSlug}/${fullSlug.packageSlug} not found`);
    }
    const renderedInputs = inputsToFQSNs(inputs || {}, fullSlug);
    // Render template variables
    const templatedYaml = renderTemplateData(rawYaml, {
        inputs: renderedInputs,
        secrets: extractFQSNMap(rawYaml, [fullSlug]),
    });
    const parsedYaml = parseBlock(templatedYaml);
    return parsedYaml;
}
function inputsToFQSNs(inputs, blockSlug) {
    const renderedInputs = {};
    for (const [key, value] of Object.entries(inputs)) {
        const stringValue = typeof value === 'string' ? value : String(value);
        renderedInputs[key] = renderTemplateData(stringValue, {
            secrets: extractFQSNMap(stringValue, [blockSlug]),
        });
    }
    return renderedInputs;
}
export function mergeOverrides(block, overrides) {
    for (const key in overrides) {
        if (overrides.hasOwnProperty(key)) {
            block[key] = overrides[key];
        }
    }
    return block;
}
