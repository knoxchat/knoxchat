import { RequestOptions } from "knoxdev-knox-config-types";
import { RequestInit, Response } from "node-fetch";
export declare function fetchwithRequestOptions(url_: URL | string, init?: RequestInit, requestOptions?: RequestOptions): Promise<Response>;
