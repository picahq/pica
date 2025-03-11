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
            scope: 'offline_access files.readwrite.all',
            refresh_token,
            grant_type: 'refresh_token',
        };

        const response = await axios({
            url: 'https://login.microsoftonline.com/common/oauth2/v2.0/token',
            method: 'POST',
            headers: {
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
                scope,
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
        throw new Error(
            `Error fetching refresh token for OneDrive: ${error}`,
        );
    }
};
