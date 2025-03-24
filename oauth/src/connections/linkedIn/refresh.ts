import { DataObject, OAuthResponse } from '../../lib/types';

export const refresh = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            OAUTH_METADATA: {
                accessToken,
                refreshToken,
                expiresIn,
                tokenType,
                meta: { scope },
            },
        } = body;

        return {
            accessToken,
            refreshToken,
            expiresIn,
            tokenType,
            meta: {
                scope,
            },
        };
    } catch (error) {
        throw new Error(`Error fetching refresh token for LinkedIn: ${error}`);
    }
};
