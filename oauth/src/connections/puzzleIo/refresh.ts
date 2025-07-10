import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';

export const refresh = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            OAUTH_CLIENT_ID: client_id,
            OAUTH_CLIENT_SECRET: client_secret,
            OAUTH_REFRESH_TOKEN: refresh_token,
            OAUTH_REQUEST_PAYLOAD: { redirectUri: redirect_uri },
            OAUTH_METADATA: { meta },
        } = body;

        const { environment } = meta;

        const baseUrl: string =
            environment === 'test'
                ? 'https://staging.southparkdata.com'
                : 'https://api.puzzle.io';

        const response = await axios({
            url: `${baseUrl}/oauth/token`,
            method: 'POST',
            data: {
                grant_type: 'refresh_token',
                refresh_token,
                client_id,
                client_secret,
                redirect_uri,
            },
        });

        const {
            access_token: accessToken,
            token_type: tokenType,
            expires_in: expiresIn,
        } = response.data;

        return {
            accessToken,
            refreshToken: refresh_token,
            expiresIn,
            tokenType: tokenType === 'bearer' ? 'Bearer' : tokenType,
            meta,
        };
    } catch (error) {
        throw new Error(`Error fetching access token for Puzzle.io: ${error}`);
    }
};
