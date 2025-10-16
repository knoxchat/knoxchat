export function mergePackages(current, incoming) {
    return {
        ...current,
        models: [...(current.models ?? []), ...(incoming.models ?? [])],
        context: [...(current.context ?? []), ...(incoming.context ?? [])],
        data: [...(current.data ?? []), ...(incoming.data ?? [])],
        mcpServers: [...(current.mcpServers ?? []), ...(incoming.mcpServers ?? [])],
        rules: [...(current.rules ?? []), ...(incoming.rules ?? [])],
        prompts: [...(current.prompts ?? []), ...(incoming.prompts ?? [])],
    };
}
