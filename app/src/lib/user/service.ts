import type { CreateUserResp } from "$lib/protos/user/proto/CreateUserResp";
import type { CreateUserReq } from "$lib/protos/user/proto/CreateUserReq";
import type { GetUserResp } from "$lib/protos/user/proto/GetUserResp";

// TODO: Put to some config.
const serverUrl = 'http://0.0.0.0:3000';


type FetchType = (input: RequestInfo, init?: RequestInit) => Promise<Response>;

export class UserService {
	constructor(fetch: FetchType) {
		this.fetch = fetch;
	}

	fetch: FetchType;

	async createUser(google_id: string | undefined): Promise<CreateUserResp> {
		const request: CreateUserReq = {
			google_id: google_id,
		};
		const response = await this.fetch(`${serverUrl}/user`, {
			method: 'POST',
			body: JSON.stringify(request),
			headers: { 'Content-Type': 'application/json' },
		});
		if (!response.ok) {
			throw new Error(`failed to create user: ${response.statusText}`)
		}
		const data: CreateUserResp = await response.json();
		return data;
	}

	async getUserByGoogleId(google_id: string | undefined): Promise<GetUserResp | undefined> {
		const response = await this.fetch(`${serverUrl}/user/google/${google_id}`);
		if (response.status === 404) {
			// user not found, return undefined
			return undefined;
		}
		if (!response.ok) {
			throw new Error(`failed to get user: ${response.statusText}`)
		}
		const data: GetUserResp = await response.json();
		return data;
	}
}
