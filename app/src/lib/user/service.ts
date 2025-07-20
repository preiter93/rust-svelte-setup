import { PUBLIC_API_URL } from "$env/static/public";
import { HttpError } from "$lib/errors";
import type { CreateUserResp } from "$lib/protos/user/proto/CreateUserResp";
import type { CreateUserReq } from "$lib/protos/user/proto/CreateUserReq";
import type { GetUserResp } from "$lib/protos/user/proto/GetUserResp";
import type { GetUserIdFromGoogleIdResp } from "$lib/protos/user/proto/GetUserIdFromGoogleIdResp";
import type { User } from "$lib/protos/user/proto/User";
import { BaseService, type FetchType } from "$lib/service";

export class UserService extends BaseService {
	constructor(fetch: FetchType) {
		super(fetch);
	}

	async createUser(
		name: string,
		email: string,
		picture: string | undefined,
		google_id: string | undefined): Promise<CreateUserResp> {
		const request: CreateUserReq = {
			name: name,
			email: email,
			picture: picture,
			google_id: google_id,
		};
		const response = await this.fetch(`${PUBLIC_API_URL}/user`, {
			method: 'POST',
			body: JSON.stringify(request),
			headers: { 'Content-Type': 'application/json' },
		}, false);
		if (!response.ok) {
			throw new Error(`failed to create user: ${response.statusText}`)
		}
		const data: CreateUserResp = await response.json();
		return data;
	}

	async getCurrentUser(): Promise<User | undefined> {
		const token = localStorage.getItem('sessionToken');
		const response = await this.fetch(`${PUBLIC_API_URL}/user/me`, {
			headers: {
				'Authorization': `Bearer ${token}`,
				'Content-Type': 'application/json',
			},
		});
		if (response.status === 404) {
			return undefined;
		}
		if (!response.ok) {
			throw new HttpError(`failed to get current user: ${response.statusText}`, response.status);
		}
		const data: GetUserResp = await response.json();
		return data.user ?? undefined;
	}

	async getUserIdFromGoogleId(google_id: string): Promise<GetUserIdFromGoogleIdResp | undefined> {
		const response = await this.fetch(`${PUBLIC_API_URL}/user/google/${google_id}`, {}, false);
		if (response.status === 404) {
			return undefined;
		}
		if (!response.ok) {
			throw new Error(`failed to get user by google id: ${response.statusText}`)
		}
		const data: GetUserIdFromGoogleIdResp = await response.json();
		return data;
	}
}
