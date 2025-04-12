import Handlebars from 'handlebars';

interface ConnectionOAuthDefinition {
    _id: string;
    configuration: OAuthApiConfig;
    connectionPlatform: string;
    compute: OAuthCompute;
    isFullTemplateEnabled?: boolean;
}

interface OAuthApiConfig {
    init: ApiModelConfig;
    refresh: ApiModelConfig;
}

interface OAuthCompute {
    init: ComputeRequest;
    refresh: ComputeRequest;
}

interface ComputeRequest {
    computation?: Compute;
    response: Compute;
}

interface Compute {
    entry: string;
    function: string;
}

const compute = async (
    payload: unknown,
    compute: Compute,
): Promise<unknown> => {
    const script = compute.function;
    const entryPoint = compute.entry;

    try {
        // eslint-disable-next-line @typescript-eslint/ban-types
        let fn: Function;

        if (
            script.endsWith('.js') ||
            script.startsWith('http://') ||
            script.startsWith('https://')
        ) {
            const module = await import(script);
            fn = module[entryPoint];
        } else {
            const wrappedCode = `
              return (function() {
                  ${script}
                  return ${entryPoint};
              })();
          `;
            fn = new Function(wrappedCode)();
        }

        if (typeof fn !== 'function') {
            throw new Error(`Entry point "${entryPoint}" is not a function`);
        }

        return await (fn as (payload: unknown) => Promise<unknown>)(payload);
    } catch (error) {
        console.error('Error in compute:', error);
        throw error;
    }
};

interface ApiModelConfig {
    baseUrl: string;
    path: string;
    headers?: Record<string, string | string[]>;
    queryParams?: { [key: string]: string };
    content?: ContentType;
}

enum ContentType {
    Json = 'json',
    Form = 'form',
}

interface OAuthPayload {
    clientId: string;
    clientSecret: string;
    metadata?: Record<string, unknown>;
}

interface OAuthResponse {
    accessToken: string;
    expiresIn: number;
    refreshToken?: string;
    tokenType?: string;
}

const headers = async (
    conn_oauth_def: ConnectionOAuthDefinition,
    computationResult?: Record<string, unknown>,
): Promise<Headers> => {
    const configHeaders = conn_oauth_def.configuration.init.headers;

    if (!configHeaders) {
        return new Headers();
    }

    try {
        const headersObj: Record<string, string> = {};
        for (const [key, value] of Object.entries(configHeaders)) {
            headersObj[key] = Array.isArray(value) ? value.join(', ') : value;
        }

        const computationPayload = computationResult?.headers as
            | Record<string, unknown>
            | undefined;

        const headersStr = JSON.stringify(headersObj);
        const template = Handlebars.compile(headersStr);
        const rendered = template(computationPayload || {});
        const renderedHeaders: Record<string, string | string[]> =
            JSON.parse(rendered);

        const resultHeaders = new Headers();
        for (const [key, value] of Object.entries(renderedHeaders)) {
            if (!key || typeof value !== 'string') {
                throw new Error(`Invalid header: ${key}`);
            }
            resultHeaders.append(key, value);
        }

        return resultHeaders;
    } catch (error) {
        console.error('Error in headers:', error);
        throw new Error(
            `Failed to process headers: ${
                error instanceof Error ? error.message : 'Unknown error'
            }`,
        );
    }
};

const query = async (
    conn_oauth_def: ConnectionOAuthDefinition,
    computationResult?: Record<string, unknown>,
): Promise<Record<string, string> | undefined> => {
    const queryParams = conn_oauth_def.configuration.init.queryParams;

    if (!queryParams) {
        return undefined;
    }

    try {
        const queryParamsObj: Record<string, string> = { ...queryParams };
        const computationPayload = computationResult?.queryParams as
            | Record<string, unknown>
            | undefined;

        const queryParamsStr = JSON.stringify(queryParamsObj);
        const template = Handlebars.compile(queryParamsStr);
        const rendered = template(computationPayload || {});
        return JSON.parse(rendered);
    } catch (error) {
        console.error('Error in query:', error);
        throw new Error(
            `Failed to process query params: ${
                error instanceof Error ? error.message : 'Unknown error'
            }`,
        );
    }
};

const body = async (
    serializedPayload: Record<string, unknown>,
    computationResult?: Record<string, unknown>,
): Promise<unknown | undefined> => {
    if (!computationResult) {
        return undefined;
    }

    try {
        const bodyObj = computationResult.body;
        if (!bodyObj) {
            return undefined;
        }

        const bodyStr = JSON.stringify(bodyObj);
        const template = Handlebars.compile(bodyStr);
        const rendered = template(serializedPayload);
        return JSON.parse(rendered);
    } catch (error) {
        console.error('Error in body:', error);
        throw new Error(
            `Failed to process body: ${
                error instanceof Error ? error.message : 'Unknown error'
            }`,
        );
    }
};

const buildRequest = async (
    conn_oauth_def: ConnectionOAuthDefinition,
    payload: OAuthPayload,
): Promise<Request> => {
    try {
        const serializedPayload: Record<string, unknown> = JSON.parse(
            JSON.stringify(payload),
        );

        const script = conn_oauth_def.compute.init.computation;
        let computationResult: Record<string, unknown> | undefined;
        if (script) {
            computationResult = (await compute(
                serializedPayload,
                script, // Changed from script[0]
            )) as Record<string, unknown> | undefined;
        }

        const headersResult = await headers(conn_oauth_def, computationResult);
        const queryResult = await query(conn_oauth_def, computationResult);
        const bodyResult = await body(serializedPayload, computationResult);

        const baseUrl = conn_oauth_def.configuration.init.baseUrl;
        const path = conn_oauth_def.configuration.init.path;
        const normalizedBase = baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`;
        const normalizedPath = path.startsWith('/') ? path.slice(1) : path;
        const urlObj = new URL(normalizedPath, normalizedBase);
        let url = urlObj.toString();

        if (queryResult) {
            const searchParams = new URLSearchParams();
            for (const [key, value] of Object.entries(queryResult)) {
                searchParams.append(key, value);
            }
            url += `?${searchParams.toString()}`;
        }

        const requestInit: RequestInit = {
            method: 'POST',
            headers: headersResult,
        };

        const contentType = conn_oauth_def.configuration.init.content;
        if (bodyResult !== undefined) {
            if (contentType === ContentType.Json) {
                requestInit.body = JSON.stringify(bodyResult);
                if (!headersResult.has('Content-Type')) {
                    headersResult.set('Content-Type', 'application/json');
                }
            } else if (contentType === ContentType.Form) {
                if (typeof bodyResult === 'object' && bodyResult !== null) {
                    const formData = new FormData();
                    for (const [key, value] of Object.entries(bodyResult)) {
                        formData.append(key, String(value));
                    }
                    requestInit.body = formData;
                } else {
                    throw new Error('Form body must be an object');
                }
            } else {
                requestInit.body = JSON.stringify(bodyResult);
                if (!headersResult.has('Content-Type')) {
                    headersResult.set('Content-Type', 'application/json');
                }
            }
        }

        return new Request(url, requestInit);
    } catch (error) {
        console.error('Error in buildRequest:', error);
        throw new Error(
            `Failed to build request: ${
                error instanceof Error ? error.message : 'Unknown error'
            }`,
        );
    }
};

const executeOAuthRequest = async (
    conn_oauth_def: ConnectionOAuthDefinition,
    payload: OAuthPayload,
): Promise<OAuthResponse> => {
    try {
        const request = await buildRequest(conn_oauth_def, payload);

        const response = await fetch(request);
        if (!response.ok) {
            throw new Error(
                `HTTP error: ${response.status} ${response.statusText}`,
            );
        }
        const jsonResponse: unknown = await response.json();

        const responseScript = conn_oauth_def.compute.init.response;
        const decodedResponse = await compute(jsonResponse, responseScript); // Changed from responseScript[0]

        if (
            !decodedResponse ||
            typeof decodedResponse !== 'object' ||
            !('accessToken' in decodedResponse) ||
            typeof decodedResponse.accessToken !== 'string' ||
            !('expiresIn' in decodedResponse) ||
            typeof decodedResponse.expiresIn !== 'number'
        ) {
            throw new Error('Invalid OAuthResponse format');
        }

        return decodedResponse as OAuthResponse;
    } catch (error) {
        console.error('Error in executeOAuthRequest:', error);
        throw new Error(
            `Failed to execute OAuth request: ${
                error instanceof Error ? error.message : 'Unknown error'
            }`,
        );
    }
};

type OAuthRequest = {
    connectionOAuthDefinition: ConnectionOAuthDefinition;
    payload: {
        clientId: string;
        clientSecret: string;
        metadata: Record<string, unknown>;
    };
    secret: {
        clientId: string;
        clientSecret: string;
    };
};

type NestedOAuthRequest = {
    body: OAuthRequest;
};

export const init = async ({
    body,
}: NestedOAuthRequest): Promise<OAuthResponse> => {
    const oauthPayload = {
        clientId: body.secret.clientId,
        clientSecret: body.secret.clientSecret,
        metadata: body.payload.metadata,
    };

    return await executeOAuthRequest(
        body.connectionOAuthDefinition,
        oauthPayload,
    );
};
