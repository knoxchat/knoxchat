import { encodePackageSlug } from "./slugs.js";
export var SecretType;
(function (SecretType) {
    SecretType["User"] = "user";
    SecretType["Package"] = "package";
    SecretType["Organization"] = "organization";
    SecretType["NotFound"] = "not_found";
    SecretType["ModelsAddOn"] = "models_add_on";
})(SecretType || (SecretType = {}));
export function encodeSecretLocation(secretLocation) {
    if (secretLocation.secretType === SecretType.Organization) {
        return `${SecretType.Organization}:${secretLocation.orgSlug}/${secretLocation.secretName}`;
    }
    else if (secretLocation.secretType === SecretType.User) {
        return `${SecretType.User}:${secretLocation.userSlug}/${secretLocation.secretName}`;
    }
    else if (secretLocation.secretType === SecretType.Package) {
        return `${SecretType.Package}:${encodePackageSlug(secretLocation.packageSlug)}/${secretLocation.secretName}`;
    }
    else if (secretLocation.secretType === SecretType.NotFound) {
        return `${SecretType.NotFound}:${secretLocation.secretName}`;
    }
    else if (secretLocation.secretType === SecretType.ModelsAddOn) {
        return `${SecretType.ModelsAddOn}:${encodePackageSlug(secretLocation.blockSlug)}/${secretLocation.secretName}`;
    }
    else {
        throw new Error(`Invalid secret type: ${secretLocation}`);
    }
}
export function decodeSecretLocation(secretLocation) {
    const [secretType, rest] = secretLocation.split(":");
    const parts = rest.split("/");
    const secretName = parts[parts.length - 1];
    switch (secretType) {
        case SecretType.Organization:
            return {
                secretType: SecretType.Organization,
                orgSlug: parts[0],
                secretName,
            };
        case SecretType.User:
            return {
                secretType: SecretType.User,
                userSlug: parts[0],
                secretName,
            };
        case SecretType.Package:
            return {
                secretType: SecretType.Package,
                packageSlug: { ownerSlug: parts[0], packageSlug: parts[1] },
                secretName,
            };
        case SecretType.NotFound:
            return {
                secretType: SecretType.NotFound,
                secretName,
            };
        case SecretType.ModelsAddOn:
            return {
                secretType: SecretType.ModelsAddOn,
                secretName,
                blockSlug: {
                    ownerSlug: parts[0],
                    packageSlug: parts[1],
                },
            };
        default:
            throw new Error(`Invalid secret type: ${secretType}`);
    }
}
