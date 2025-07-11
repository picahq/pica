import axios from 'axios';
import qs from 'qs';
import { DataObject, OAuthResponse } from '../../lib/types';

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            clientId: client_id,
            clientSecret: client_secret,
            metadata: {
                code,
                redirectUri: redirect_uri,
                formData: { CONTENTSTACK_CONTENT_MANAGEMENT_STACK_API_KEY },
            },
        } = body;

        const requestBody = {
            grant_type: 'authorization_code',
            code,
            client_id,
            client_secret,
            redirect_uri,
        };

        const response = await axios({
            url: 'https://app.contentstack.com/apps-api/token',
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
            organization_uid: organizationUid,
        } = response.data;

        await axios({
            url: 'https://api.contentstack.io/v3/stacks/settings',
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                Authorization: `Bearer ${accessToken}`,
                api_key: CONTENTSTACK_CONTENT_MANAGEMENT_STACK_API_KEY,
            },
        });

        return {
            accessToken,
            refreshToken,
            expiresIn,
            tokenType: tokenType === 'bearer' ? 'Bearer' : tokenType,
            meta: {
                organizationUid,
                stackApiKey: CONTENTSTACK_CONTENT_MANAGEMENT_STACK_API_KEY,
            },
        };
    } catch (error) {
        throw new Error(
            `Error fetching access token for Contentstack Content Management: ${error}`,
        );
    }
};
