export function llms(provider, infos) {
    return infos.map((info) => ({
        ...info,
        provider,
    }));
}
