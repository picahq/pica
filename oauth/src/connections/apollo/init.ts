import axios from "axios";
import { DataObject, OAuthResponse } from "../../lib/types";

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
	try {
		const {
			clientId: client_id,
			clientSecret: client_secret,
			metadata: {
				code,
				redirectUri: redirect_uri,
			} = {},
		} = body;

		const data = {
			client_id,
			client_secret,
			code,
			redirect_uri,
			grant_type: "authorization_code",
		};

		const response = await axios({
			url: "https://app.apollo.io/api/v1/oauth/token",
			method: "POST",
			headers: {
				Accept: "application/json",
			},
			data,
		});

		const {
			data: {
				access_token: accessToken,
				refresh_token: refreshToken,
				token_type: tokenType,
				expires_in: expiresIn
			},
		} = response;

		return {
			accessToken,
			refreshToken,
			expiresIn,
			tokenType,
			meta: {},
		};
	} catch (error) {
		throw new Error(`Error fetching access token for Apollo: ${error}`);
	}
}; 