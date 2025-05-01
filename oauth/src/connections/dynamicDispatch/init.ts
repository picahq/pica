import Handlebars from 'handlebars';

interface ConnectionOAuthDefinition {
    id: number;
    connectionDefId: number;
    connectionPlatform: string;
    initCompute: string;
    callbackInitCompute: string;
    refreshCompute: string;
    callbackRefreshCompute: string;
    frontend: Frontend;
    templatedInFull: boolean;
    createdAt: string;
    updatedAt: string;
    version: string;
    deleted: boolean;
    changeLog: Record<string, unknown>;
    tags: string[];
    active: boolean;
    initEndpointInfo: EndpointInfo;
    refreshEndpointInfo: EndpointInfo;
}

interface Frontend {
    platformRedirectUri: string;
    sandboxPlatformRedirectUri: string;
    scopes: string;
    iosRedirectUri: string;
    separator: string;
}

interface EndpointInfo {
    authMethod: AuthMethod;
    baseUrl: string;
    path: string;
    headers: Record<string, string>;
    queryParams: Record<string, string>;
    content: ContentType;
}

interface AuthMethod {
    authMethodType: string;
    key: string;
    value: string;
    username: string;
    password: string;
    hashAlgorithm: string;
    realm: string;
}

enum ContentType {
    Json = 'json',
    Form = 'form',
}

const compute = async (payload: unknown, script: string): Promise<unknown> => {
    const entryPoint = 'compute';

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

interface OAuthPayload {
    clientId: string;
    clientSecret: string;
    metadata?: Record<string, unknown>;
    grant_type?: string;
    code?: string;
    code_verifier?: string;
}

interface OAuthResponse {
    access_token: string;
    expires_in: number;
    refresh_token?: string;
    token_type?: string;
    metadata?: Record<string, unknown>;
}

const headers = async (
    conn_oauth_def: ConnectionOAuthDefinition,
    computationResult?: Record<string, unknown>,
): Promise<Headers> => {
    const configHeaders = conn_oauth_def.initEndpointInfo.headers;

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
    const queryParams = conn_oauth_def.initEndpointInfo.queryParams;

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
    payload: OAuthPayload,
    connOauthDef: ConnectionOAuthDefinition,
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
        const parsedBody = JSON.parse(rendered);
        parsedBody.redirect_uri = connOauthDef.frontend.iosRedirectUri;
        const grantType = payload.grant_type;
        const code = payload.code;
        const codeVerifier = payload.code_verifier;
        parsedBody.grant_type = grantType;
        parsedBody.code = code;
        parsedBody.code_verifier = codeVerifier;
        parsedBody.metadata = payload.metadata;

        // for backward compatibility
        parsedBody.metadata.code = code;
        parsedBody.metadata.redirectUri = connOauthDef.frontend.iosRedirectUri;

        return parsedBody;
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

        const script = conn_oauth_def.initCompute;
        let computationResult: Record<string, unknown> | undefined;
        if (script) {
            computationResult = (await compute(
                serializedPayload,
                script, // Changed from script[0]
            )) as Record<string, unknown> | undefined;
        }

        const headersResult = await headers(conn_oauth_def, computationResult);
        const queryResult = await query(conn_oauth_def, computationResult);
        const bodyResult = await body(
            payload,
            conn_oauth_def,
            serializedPayload,
            computationResult,
        );

        const baseUrl = conn_oauth_def.initEndpointInfo.baseUrl;
        const path = conn_oauth_def.initEndpointInfo.path;
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

        const contentType = conn_oauth_def.initEndpointInfo.content;
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

        const responseScript = conn_oauth_def.callbackInitCompute;
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

        const transformedResponse: OAuthResponse = {
            access_token: decodedResponse.accessToken,
            expires_in: decodedResponse.expiresIn,
            refresh_token: decodedResponse.accessToken,
            token_type: 'bearer',
        };

        if (
            decodedResponse &&
            'refreshToken' in decodedResponse &&
            typeof decodedResponse.refreshToken === 'string'
        ) {
            transformedResponse.refresh_token = decodedResponse.refreshToken;
        }

        if (
            decodedResponse &&
            'tokenType' in decodedResponse &&
            typeof decodedResponse.tokenType === 'string'
        ) {
            transformedResponse.token_type = decodedResponse.tokenType;
        }

        if (
            decodedResponse &&
            'meta' in decodedResponse &&
            typeof decodedResponse.meta === 'object'
        ) {
            transformedResponse.metadata =
                decodedResponse.meta as unknown as Record<string, unknown>;
        }

        if (
            decodedResponse &&
            'metadata' in decodedResponse &&
            typeof decodedResponse.metadata === 'object'
        ) {
            transformedResponse.metadata =
                decodedResponse.metadata as unknown as Record<string, unknown>;
        }

        return transformedResponse;
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
    grant_type: string;
    code: string;
    code_verifier: string;
    redirect_uri: string;
    connection_oauth_payload: string;
    payload: string;
};

type Payload = {
    clientId: string;
    clientSecret: string;
    metadata: Record<string, unknown>;
};

type NestedOAuthRequest = {
    body: OAuthRequest;
};

export const init = async ({
    body,
}: NestedOAuthRequest): Promise<OAuthResponse> => {
    const connectionOAuthDefinition: ConnectionOAuthDefinition = JSON.parse(
        body.connection_oauth_payload,
    );
    const payload: Payload = JSON.parse(body.payload);

    const oauthPayload = {
        clientId: payload.clientId,
        clientSecret: payload.clientSecret,
        metadata: payload.metadata,
        grant_type: body.grant_type,
        code: body.code,
        code_verifier: body.code_verifier,
    };

    return await executeOAuthRequest(connectionOAuthDefinition, oauthPayload);
};
