import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            clientId: client_id,
            clientSecret: client_secret,
            metadata: { code, redirectUri: redirect_uri },
        } = body;

        const data = {
            grant_type: 'authorization_code',
            code,
            client_id,
            client_secret,
            redirect_uri,
        };

        const response = await axios({
            url: 'https://api.webflow.com/oauth/access_token',
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                Accept: 'application/json',
            },
            data,
        });

        const { access_token: accessToken, token_type: tokenType } =
            response.data;

        return {
            accessToken,
            refreshToken: accessToken,
            expiresIn: 2147483647,
            tokenType: tokenType === 'bearer' ? 'Bearer' : tokenType,
            meta: {},
        };
    } catch (error) {
        throw new Error(`Error fetching access token for Webflow: ${error}`);
    }
};
