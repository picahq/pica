import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            clientId,
            clientSecret,
            metadata: { code, redirectUri },
        } = body;

        const requestBody = {
            grant_type: 'authorization_code',
            code: code,
            client_id: clientId,
            client_secret: clientSecret,
            redirect_uri: redirectUri,
        };

        const response = await axios.postForm(
            'https://login.mailchimp.com/oauth2/token',
            requestBody,
        );

        const {
            data: { access_token: accessToken },
        } = response;

        const metadataResponse = await axios.get(
            'https://login.mailchimp.com/oauth2/metadata',
            {
                headers: {
                    Authorization: `OAuth ${accessToken}`,
                },
            },
        );

        const {
            data: { dc },
        } = metadataResponse;

        return {
            accessToken,
            refreshToken: accessToken,
            expiresIn: 2147483647,
            tokenType: 'Bearer',
            meta: {
                serverPrefix: dc,
            },
        };
    } catch (error) {
        throw new Error(
            `Error fetching access token for Mailchimp Marketing: ${error}`,
        );
    }
};
