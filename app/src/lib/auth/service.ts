import { PUBLIC_API_URL } from "$env/static/public";

import type { CreateSessionReq } from "$lib/protos/auth/proto/CreateSessionReq";
import type { CreateSessionResp } from "$lib/protos/auth/proto/CreateSessionResp";
import { Google } from "arctic";
import { PUBLIC_GOOGLE_CLIENT_ID, PUBLIC_GOOGLE_CLIENT_SECRET } from "$env/static/public";
import { BaseService, type FetchType } from "$lib/service";
import type { StartGoogleLoginReq } from "$lib/protos/auth/proto/StartGoogleLoginReq";
import type { StartGoogleLoginResp } from "$lib/protos/auth/proto/StartGoogleLoginResp";


export const google = new Google(PUBLIC_GOOGLE_CLIENT_ID, PUBLIC_GOOGLE_CLIENT_SECRET, "http://localhost:5173/login/google/callback");

export class AuthService extends BaseService {
	constructor(fetch: FetchType) {
		super(fetch);
	}

	async createSession(user_id: string | undefined): Promise<CreateSessionResp> {
		const request: CreateSessionReq = {
			user_id: user_id,
		};
		const response = await this.fetch(`${PUBLIC_API_URL}/session`, {
			method: "POST",
			body: JSON.stringify(request),
			headers: { 'Content-Type': 'application/json' },
		}, false);
		if (!response.ok) {
			throw new Error(`failed to create session: ${response.statusText}`)
		}
		const data: CreateSessionResp = await response.json();
		return data;
	}

	async startGoogleLogin(): Promise<StartGoogleLoginResp> {
		const response = await this.fetch(`${PUBLIC_API_URL}/auth/google/login`, {
			credentials: 'include',
		}, false);
		if (!response.ok) {
			throw new Error(`failed to start google login: ${response.statusText}`)
		}
		const data: StartGoogleLoginResp = await response.json();
		return data;
	}

	async handleGoogleCallback(state: string, code: string): Promise<StartGoogleLoginResp> {
		const response = await this.fetch(`${PUBLIC_API_URL}/auth/google/callback?state=${state}&code=${code}`, {
			credentials: 'include',
		}, false);
		if (!response.ok) {
			throw new Error(`failed to handle google callback: ${response.statusText}`)
		}
		const data: StartGoogleLoginResp = await response.json();
		return data;
	}
}
