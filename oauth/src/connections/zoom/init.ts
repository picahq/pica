import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';
import qs from 'qs';
import { base64UrlEncode } from '../../lib/helpers';

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const requestBody = {
            grant_type: 'authorization_code',
            code: body.metadata?.code,
            client_id: body.clientId,
            client_secret: body.clientSecret,
            redirect_uri: body.metadata?.redirectUri,
        };

        const authorizationToken = await base64UrlEncode(
            `${body.clientId}:${body.clientSecret}`,
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
        throw new Error(`Error fetching access token for Zoom: ${error}`);
    }
};
