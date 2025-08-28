import { PUBLIC_API_URL } from "$env/static/public";

import { BaseService, type FetchType } from "$lib/service";



export class AuthService extends BaseService {
	constructor(fetch: FetchType) {
		super(fetch);
	}

	async handleOauthCallback(provider: string, state: string, code: string): Promise<void> {
		const response = await this.fetch(`${PUBLIC_API_URL}/auth/${provider}/callback?state=${state}&code=${code}`);
		if (!response.ok) {
			throw new Error(`failed to handle oauth callback: ${response.statusText}`)
		}
	}
}
