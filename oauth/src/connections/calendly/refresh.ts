import { DataObject, OAuthResponse } from "../../lib/types";
import axios from "axios";

export const base64UrlEncode = async (input: string) => {
	const byteArray = new TextEncoder().encode(input);

	let base64String = btoa(String.fromCharCode(...byteArray));

	base64String = base64String.replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");

	while (base64String.length % 4 !== 0) {
		base64String += "=";
	}

	return base64String;
};

export const refresh = async ({ body }: DataObject): Promise<OAuthResponse> => {
	try {
		const {
			OAUTH_CLIENT_ID: client_id,
			OAUTH_CLIENT_SECRET: client_secret,
			OAUTH_REFRESH_TOKEN: refresh_token,
		} = body;

		const data = {
			refresh_token,
			grant_type: "refresh_token",
		};

		const authorizationToken = await base64UrlEncode(`${client_id}:${client_secret}`);

		const response = await axios({
			url: "https://auth.calendly.com/oauth/token",
			method: "POST",
			headers: {
				"Content-Type": "application/x-www-form-urlencoded",
				Authorization: `Basic ${authorizationToken}`
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
		throw new Error(`Error fetching refresh token for Calendly: ${error}`);
	}
}; 