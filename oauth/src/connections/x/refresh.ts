import { DataObject, OAuthResponse } from '../../lib/types';
import axios from 'axios';

export const refresh = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            OAUTH_CLIENT_ID: client_id,
            OAUTH_CLIENT_SECRET: client_secret,
            OAUTH_REFRESH_TOKEN: refresh_token,
        } = body;

        const data = {
            client_id,
            client_secret,
            refresh_token,
            grant_type: 'refresh_token',
        };

        const response = await axios({
            url: 'https://api.x.com/2/oauth2/token',
            method: 'POST',
            headers: {
                Accept: 'application/json',
                'Content-Type': 'application/x-www-form-urlencoded',
            },
            data,
        });

        const {
            data: {
                access_token: accessToken,
                refresh_token: refreshToken,
                token_type: tokenType,
                expires_in: expiresIn,
            },
        } = response;

        return {
            accessToken,
            refreshToken,
            expiresIn,
            tokenType,
            meta: {},
        };
    } catch (error) {
        throw new Error(`Error fetching refresh token for X: ${error}`);
    }
};
