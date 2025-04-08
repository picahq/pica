import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';
import qs from 'qs';

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            clientId: client_id,
            clientSecret: client_secret,
            metadata: { code, redirectUri: redirect_uri } = {},
        } = body;

        const data = qs.stringify({
            client_id,
            client_secret,
            code,
            redirect_uri,
            grant_type: 'authorization_code',
            code_verifier:
                '2a67864e439b338f40487eef9ffcf11d5f785981dc58a568cd1cf005',
        });

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
            tokenType: tokenType === 'bearer' ? 'Bearer' : tokenType,
            meta: {},
        };
    } catch (error) {
        throw new Error(`Error fetching access token for X: ${error}`);
    }
};
