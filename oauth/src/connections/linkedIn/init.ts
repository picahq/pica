import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            clientId: client_id,
            clientSecret: client_secret,
            metadata: { code, redirectUri: redirect_uri } = {},
        } = body;

        const data = {
            client_id,
            client_secret,
            code,
            redirect_uri,
            grant_type: 'authorization_code',
        };

        const response = await axios({
            url: 'https://www.linkedin.com/oauth/v2/accessToken',
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
            },
            data,
        });

        const {
            data: { access_token: accessToken, expires_in: expiresIn, scope },
        } = response;

        return {
            accessToken,
            refreshToken: accessToken,
            expiresIn,
            tokenType: 'Bearer',
            meta: {
                scope,
            },
        };
    } catch (error) {
        throw new Error(`Error fetching access token for LinkedIn: ${error}`);
    }
};
