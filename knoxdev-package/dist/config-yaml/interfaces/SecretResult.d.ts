import { FQSN, PackageSlug } from "./slugs.js";
export declare enum SecretType {
    User = "user",
    Package = "package",
    Organization = "organization",
    NotFound = "not_found",
    ModelsAddOn = "models_add_on"
}
export interface OrgSecretLocation {
    secretType: SecretType.Organization;
    orgSlug: string;
    secretName: string;
}
export interface PackageSecretLocation {
    secretType: SecretType.Package;
    packageSlug: PackageSlug;
    secretName: string;
}
export interface UserSecretLocation {
    secretType: SecretType.User;
    userSlug: string;
    secretName: string;
}
export interface ModelsAddOnSecretLocation {
    secretType: SecretType.ModelsAddOn;
    blockSlug: PackageSlug;
    secretName: string;
}
/**
 * If not found in user/package/org secrets, then there's a chance it's in
 * - the on-prem proxy
 * - models add-on
 */
export interface NotFoundSecretLocation {
    secretType: SecretType.NotFound;
    secretName: string;
}
export type SecretLocation = OrgSecretLocation | PackageSecretLocation | UserSecretLocation | NotFoundSecretLocation | ModelsAddOnSecretLocation;
export declare function encodeSecretLocation(secretLocation: SecretLocation): string;
export declare function decodeSecretLocation(secretLocation: string): SecretLocation;
export interface NotFoundSecretResult {
    found: false;
    secretLocation: NotFoundSecretLocation;
    fqsn: FQSN;
}
export interface FoundSecretResult {
    found: true;
    secretLocation: OrgSecretLocation | PackageSecretLocation | ModelsAddOnSecretLocation;
    fqsn: FQSN;
}
export interface FoundUserSecretResult {
    found: true;
    secretLocation: UserSecretLocation;
    value: string;
    fqsn: FQSN;
}
export type SecretResult = FoundSecretResult | FoundUserSecretResult | NotFoundSecretResult;
