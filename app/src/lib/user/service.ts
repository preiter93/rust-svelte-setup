import { PUBLIC_API_URL } from "$env/static/public";
import { HttpError } from "$lib/errors";
import type { GetUserResp } from "$lib/protos/user/proto/GetUserResp";
import type { User } from "$lib/protos/user/proto/User";
import { BaseService, type FetchType } from "$lib/service";

export class UserService extends BaseService {
	constructor(fetch: FetchType) {
		super(fetch);
	}

	async getCurrentUser(): Promise<User | undefined> {
		const response = await this.fetch(`${PUBLIC_API_URL}/user/me`, {
			headers: {
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
}
