export function getBlockType(block) {
    if (block.context?.length) {
        return "context";
    }
    else if (block.models?.length) {
        return "models";
    }
    else if (block.mcpServers?.length) {
        return "mcpServers";
    }
    else if (block.data?.length) {
        return "data";
    }
    else if (block.rules?.length) {
        return "rules";
    }
    else if (block.prompts?.length) {
        return "prompts";
    }
    else {
        return undefined;
    }
}
