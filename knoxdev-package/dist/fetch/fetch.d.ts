import { RequestOptions } from "../config-types/index.js";
import { RequestInit, Response } from "node-fetch";
export declare function fetchwithRequestOptions(url_: URL | string, init?: RequestInit, requestOptions?: RequestOptions): Promise<Response>;
