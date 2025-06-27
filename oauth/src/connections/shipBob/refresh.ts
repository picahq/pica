import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';
import qs from 'qs';

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

        const url =
            environment === 'test'
                ? 'https://authstage.shipbob.com/connect/token'
                : 'https://auth.shipbob.com/connect/token';

        const requestBody = {
            grant_type: 'refresh_token',
            client_id,
            client_secret,
            redirect_uri,
            refresh_token,
        };

        const response = await axios({
            url,
            method: 'POST',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
                Accept: 'application/json',
            },
            data: qs.stringify(requestBody),
        });

        const {
            access_token: accessToken,
            refresh_token: refreshToken,
            token_type: tokenType,
            expires_in: expiresIn,
        } = response.data;

        return {
            accessToken,
            refreshToken,
            expiresIn,
            tokenType: tokenType === 'bearer' ? 'Bearer' : tokenType,
            meta,
        };
    } catch (error) {
        throw new Error(`Error fetching access token for ShipBob: ${error}`);
    }
};
