import axios from "axios";
import { DataObject, OAuthResponse } from "../../lib/types";

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
	try {
		const requestBody = {
			client_id: body.clientId,
			client_secret: body.clientSecret,
			code: body.metadata?.code,
			redirect_uri: body.metadata?.redirectUri,
		};

		const response = await axios.post(
			"https://github.com/login/oauth/access_token",
			requestBody,
			{
				headers: {
					Accept: "application/json",
				},
			},
		);

		const {
			access_token: accessToken,
			token_type: tokenType,
		} = response.data;

		return {
			accessToken,
			refreshToken: accessToken, // GitHub tokens don't need refreshing
			expiresIn: 2147483647, // Set max expiration
			tokenType: tokenType === "bearer" ? "Bearer" : tokenType,
			meta: {},
		};
	} catch (error) {
		throw new Error(`Error fetching access token for GitHub: ${error}`);
	}
}; 