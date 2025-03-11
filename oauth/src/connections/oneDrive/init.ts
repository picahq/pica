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
            scope: 'offline_access files.readwrite.all',
            grant_type: 'authorization_code',
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
            `Error fetching access token for OneDrive: ${error}`,
        );
    }
};
