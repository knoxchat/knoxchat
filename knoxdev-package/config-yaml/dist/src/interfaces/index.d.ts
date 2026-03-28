import { SecretLocation, SecretResult } from "./SecretResult.js";
import { FQSN, FullSlug, PackageSlug } from "./slugs.js";
/**
 * A registry stores the content of packages
 */
export interface Registry {
    getContent(fullSlug: FullSlug): Promise<string>;
}
export type SecretNamesMap = Map<FQSN, string>;
/**
 * A secret store stores secrets
 */
export interface SecretStore {
    get(secretName: string): Promise<string | undefined>;
    set(secretName: string, secretValue: string): Promise<void>;
}
export interface PlatformClient {
    resolveFQSNs(fqsns: FQSN[]): Promise<(SecretResult | undefined)[]>;
}
export interface PlatformSecretStore {
    getSecretFromSecretLocation(secretLocation: SecretLocation): Promise<string | undefined>;
}
export declare function getLocationsToLook(assistantSlug: PackageSlug, blockSlug: PackageSlug | undefined, currentUserSlug: string, secretName: string, orgScopeSlug: string | null): SecretLocation[];
export declare function listAvailableSecrets(userSecretNames: string[], orgSecretNames: string[], assistantSecretNames: string[], blockSecretNames: string[], assistantSlug: PackageSlug, blockSlug: PackageSlug | undefined, currentUserSlug: string, orgScopeSlug: string | null): SecretLocation[];
export declare function resolveFQSN(currentUserSlug: string, fqsn: FQSN, platformSecretStore: PlatformSecretStore, orgScopeSlug: string | null): Promise<SecretResult>;
