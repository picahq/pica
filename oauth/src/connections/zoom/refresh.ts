import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';
import qs from 'qs';
import { base64UrlEncode } from '../../lib/helpers';

export const refresh = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            OAUTH_CLIENT_ID: clientId,
            OAUTH_CLIENT_SECRET: clientSecret,
            OAUTH_REFRESH_TOKEN: refresh_token,
        } = body;

        const requestBody = {
            grant_type: 'refresh_token',
            refresh_token,
        };

        const authorizationToken = await base64UrlEncode(
            `${clientId}:${clientSecret}`,
        );

        const response = await axios({
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
                Authorization: `Basic ${authorizationToken}`,
            },
            data: qs.stringify(requestBody),
            url: 'https://zoom.us/oauth/token',
        });

        const {
            data: {
                token_type: tokenType,
                scope,
                access_token: accessToken,
                expires_in: expiresIn,
                refresh_token: refreshToken,
            },
        } = response;

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
        throw new Error(`Error fetching refresh token for Zoom: ${error}`);
    }
};
