export class RegistryClient {
    accessToken;
    apiBase;
    constructor(accessToken, apiBase = "https://knox.chat/") {
        this.accessToken = accessToken;
        this.apiBase = apiBase;
        if (!this.apiBase.endsWith("/")) {
            this.apiBase += "/";
        }
    }
    async getContent(fullSlug) {
        const response = await fetch(`${this.apiBase}registry/v1/${fullSlug.ownerSlug}/${fullSlug.packageSlug}/${fullSlug.versionSlug}`, {
            headers: {
                ...(this.accessToken
                    ? { Authorization: `Bearer ${this.accessToken}` }
                    : {}),
            },
        });
        const data = await response.json();
        return data.content;
    }
}
