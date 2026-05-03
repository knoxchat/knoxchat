import { PlatformSecretStore, SecretStore } from "../interfaces/index.js";
import { SecretLocation } from "../interfaces/SecretResult.js";
export declare function resolveSecretLocationInProxy(secretLocaton: SecretLocation, platformSecretStore: PlatformSecretStore, environmentSecretStore?: SecretStore): Promise<string>;
