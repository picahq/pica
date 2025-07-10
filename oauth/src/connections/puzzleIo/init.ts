import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            clientId: client_id,
            clientSecret: client_secret,
            metadata: { code, redirectUri: redirect_uri, environment },
        } = body;

        const baseUrl: string =
            environment === 'test'
                ? 'https://staging.southparkdata.com'
                : 'https://api.puzzle.io';

        const response = await axios({
            url: `${baseUrl}/oauth/token`,
            method: 'POST',
            data: {
                grant_type: 'authorization_code',
                code,
                client_id,
                client_secret,
                redirect_uri,
            },
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
            meta: {
                environment,
                PUZZLE_BASE_URL: baseUrl,
            },
        };
    } catch (error) {
        throw new Error(`Error fetching access token for Puzzle.io: ${error}`);
    }
};
