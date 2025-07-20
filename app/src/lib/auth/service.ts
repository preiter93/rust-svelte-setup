import { PUBLIC_API_URL } from "$env/static/public";

import type { CreateSessionReq } from "$lib/protos/auth/proto/CreateSessionReq";
import type { CreateSessionResp } from "$lib/protos/auth/proto/CreateSessionResp";
import { Google } from "arctic";
import { PUBLIC_GOOGLE_CLIENT_ID, PUBLIC_GOOGLE_CLIENT_SECRET } from "$env/static/public";


export const google = new Google(PUBLIC_GOOGLE_CLIENT_ID, PUBLIC_GOOGLE_CLIENT_SECRET, "http://localhost:5173/login/google/callback");

type FetchType = (input: RequestInfo, init?: RequestInit) => Promise<Response>;
export class AuthService {
	constructor(fetch: FetchType) {
		this.fetch = fetch;
	}

	fetch: FetchType;

	async createSession(user_id: string | undefined): Promise<CreateSessionResp> {
		const request: CreateSessionReq = {
			user_id: user_id,
		};
		const response = await this.fetch(`${PUBLIC_API_URL}/session`, {
			method: "POST",
			body: JSON.stringify(request),
			headers: { 'Content-Type': 'application/json' },
		});
		if (!response.ok) {
			throw new Error(`failed to create session: ${response.statusText}`)
		}
		const data: CreateSessionResp = await response.json();
		return data;
	}
}
