import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';
import qs from 'qs';
import 'dotenv/config';
import { base64UrlEncode } from '../../lib/helpers';

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            clientId: client_id,
            clientSecret: client_secret,
            metadata: { code, redirectUri: redirect_uri } = {},
        } = body;

        const data = qs.stringify({
            code,
            redirect_uri,
            grant_type: 'authorization_code',
            code_verifier: process.env.CANVA_CODE_VERIFIER,
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
            tokenType: tokenType === 'bearer' ? 'Bearer' : tokenType,
            meta: {},
        };
    } catch (error) {
        throw new Error(`Error fetching access token for Canva: ${error}`);
    }
};
