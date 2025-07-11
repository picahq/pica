import { DataObject, OAuthResponse } from '../../lib/types';
import axios from 'axios';
import { base64UrlEncode } from '../../lib/helpers';
import qs from 'qs';

export const refresh = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            OAUTH_CLIENT_ID: client_id,
            OAUTH_CLIENT_SECRET: client_secret,
            OAUTH_REFRESH_TOKEN: refresh_token,
        } = body;

        const data = qs.stringify({
            refresh_token,
            grant_type: 'refresh_token',
        });

        const authorizationToken = await base64UrlEncode(
            `${client_id}:${client_secret}`,
        );

        const response = await axios({
            url: 'https://api.canva.com/rest/v1/oauth/token',
            method: 'POST',
            headers: {
                Accept: 'application/json',
                'Content-Type': 'application/x-www-form-urlencoded',
                Authorization: `Basic ${authorizationToken}`,
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
        throw new Error(`Error fetching refresh token for Canva: ${error}`);
    }
};
