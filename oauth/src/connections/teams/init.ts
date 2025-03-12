import axios from 'axios';
import { DataObject, OAuthResponse } from '../../lib/types';

export const init = async ({ body }: DataObject): Promise<OAuthResponse> => {
    try {
        const {
            clientId: client_id,
            clientSecret: client_secret,
            metadata: { code, redirectUri: redirect_uri } = {},
        } = body;

        const data = {
            client_id,
            client_secret,
            code,
            redirect_uri,
            scope: 'Calendars.ReadWrite Calendars.ReadWrite.Shared offline_access Team.Create Team.ReadBasic.All Channel.Create Channel.Delete.All Channel.ReadBasic.All ChannelMember.ReadWrite.All ChannelMessage.ReadWrite ChannelMessage.UpdatePolicyViolation.All ChannelSettings.ReadWrite.All Chat.ManageDeletion.All Chat.ReadWrite Chat.ReadWrite.All Chat.UpdatePolicyViolation.All ChatMessage.Send Contacts.ReadWrite Contacts.ReadWrite.Shared Mail.ReadWrite.Shared Mail.Send Mail.Send.Shared OnlineMeetings.ReadWrite TeamsActivity.Send TeamsTab.Create TeamsTab.ReadWrite.All User.ReadWrite.All VirtualEvent.ReadWrite',
            grant_type: 'authorization_code',
        };

        const response = await axios({
            url: 'https://login.microsoftonline.com/common/oauth2/v2.0/token',
            method: 'POST',
            headers: {
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
                scope,
            },
        } = response;

        return {
            accessToken,
            refreshToken,
            expiresIn,
            tokenType,
            meta: {
                scope,
            },
        };
    } catch (error) {
        throw new Error(
            `Error fetching access token for Teams: ${error}`,
        );
    }
};
