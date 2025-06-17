import axios from 'axios';
import qs from 'qs';
import { DataObject, OAuthResponse } from '../../lib/types';

export const refresh = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            OAUTH_CLIENT_ID: client_id,
            OAUTH_CLIENT_SECRET: client_secret,
            OAUTH_REFRESH_TOKEN: refresh_token,
            OAUTH_METADATA: { meta },
            OAUTH_REQUEST_PAYLOAD: { redirectUri: redirect_uri },
        } = body;

        const requestBody = {
            grant_type: 'refresh_token',
            refresh_token,
            client_id,
            client_secret,
            redirect_uri,
        };

        const response = await axios({
            url: 'https://app.asana.com/-/oauth_token',
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
                Accept: 'application/json',
            },
            data: qs.stringify(requestBody),
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
        throw new Error(`Error fetching access token for Asana: ${error}`);
    }
};
