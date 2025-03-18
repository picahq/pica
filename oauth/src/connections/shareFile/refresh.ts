import { DataObject, OAuthResponse } from '../../lib/types';
import axios from 'axios';
import qs from 'qs';

export const refresh = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            OAUTH_CLIENT_ID: client_id,
            OAUTH_CLIENT_SECRET: client_secret,
            OAUTH_REFRESH_TOKEN: refresh_token,
            OAUTH_REQUEST_PAYLOAD: {
                additionalData: { subdomain, apicp },
            },
        } = body;

        const data = qs.stringify({
            client_id,
            client_secret,
            refresh_token,
            grant_type: 'refresh_token',
        });

        const response = await axios({
            url: `https://${subdomain}.${apicp}/oauth/token `,
            method: 'POST',
            headers: {
                Accept: 'application/x-www-form-urlencoded',
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
        throw new Error(`Error fetching refresh token for ShareVine: ${error}`);
    }
};
