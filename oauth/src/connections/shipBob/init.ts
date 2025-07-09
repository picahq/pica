import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';
import qs from 'qs';

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            clientId: client_id,
            clientSecret: client_secret,
            metadata: { code, redirectUri: redirect_uri, environment },
        } = body;

        const url =
            environment === 'test'
                ? 'https://authstage.shipbob.com/connect/token'
                : 'https://auth.shipbob.com/connect/token';

        const requestBody = {
            grant_type: 'authorization_code',
            code,
            client_id,
            client_secret,
            redirect_uri,
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

        const baseUrl =
            environment === 'test'
                ? 'https://sandbox-api.shipbob.com/2.0/'
                : 'https://api.shipbob.com/2.0/';

        const channelResponse = await axios({
            url: `${baseUrl}channel`,
            method: 'GET',
            headers: {
                'Content-Type': 'application/x-www-form-urlencoded',
                Accept: 'application/json',
                Authorization: `Bearer ${accessToken}`,
            },
        });

        return {
            accessToken,
            refreshToken,
            expiresIn,
            tokenType: tokenType === 'bearer' ? 'Bearer' : tokenType,
            meta: {
                environment,
                baseUrl,
                channelId: channelResponse.data[0].id,
            },
        };
    } catch (error) {
        throw new Error(`Error fetching access token for ShipBob: ${error}`);
    }
};
