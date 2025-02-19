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
			code,
			redirect_uri,
			grant_type: "authorization_code",
		};

		const encoded = Buffer.from(`${client_id}:${client_secret}`).toString("base64");

		const response = await axios({
			url: "https://api.notion.com/v1/oauth/token",
			method: "POST",
			headers: {
				Accept: "application/json",
				"Content-Type": "application/json",
				Authorization: `Basic ${encoded}`
			},
			data: JSON.stringify(data),
		});

		console.log(JSON.stringify(response.data, null, 4));

		const {
			data: {
				access_token: accessToken,
			},
		} = response;

		return {
			accessToken,
			refreshToken: accessToken,
			expiresIn: 2147483647,
			tokenType: "Bearer",
			meta: {},
		};
	} catch (error) {
		throw new Error(`Error fetching access token for Notion: ${error}`);
	}
}; 