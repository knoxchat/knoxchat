function convertModel(m, roles) {
    return {
        name: m.title,
        provider: m.provider,
        model: m.model,
        apiKey: m.apiKey,
        apiBase: m.apiBase,
        roles,
        requestOptions: m.requestOptions,
        defaultCompletionOptions: m.completionOptions,
    };
}
function convertContext(configJson) {
    const context = configJson.contextProviders?.map((ctx) => {
        return {
            uses: `builtin/${ctx.name}`,
            with: ctx.params,
        };
    }) ?? [];
    return context;
}
function convertCustomCommand(cmd) {
    return {
        name: cmd.name,
        description: cmd.description,
        prompt: cmd.prompt,
    };
}
function convertMcp(mcp) {
    const { transport } = mcp;
    const { command, args, env } = transport;
    return {
        command,
        args,
        env,
        name: "MCP Server",
    };
}
export function convertJsonToYamlConfig(configJson) {
    // models
    const models = configJson.models.map((m) => convertModel(m, ["chat"]));
    const autocompleteModels = Array.isArray(configJson.tabAutocompleteModel)
        ? configJson.tabAutocompleteModel
        : configJson.tabAutocompleteModel
            ? [configJson.tabAutocompleteModel]
            : [];
    models.push(...autocompleteModels.map((m) => convertModel(m, ["autocomplete"])));
    // context
    const context = convertContext(configJson);
    // mcpServers
    // Types for "experimental" don't exist
    const mcpServers = configJson.experimental?.modelContextProtocolServers?.map(convertMcp);
    // prompts
    const prompts = configJson.customCommands?.map(convertCustomCommand);
    const configYaml = {
        name: "Knox Config",
        version: "0.0.1",
        models,
        context,
        rules: configJson.systemMessage ? [configJson.systemMessage] : undefined,
        prompts,
        mcpServers,
    };
    return configYaml;
}
