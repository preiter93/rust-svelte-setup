import { goto } from '$app/navigation';

export type FetchType = typeof fetch;

export class BaseService {
	protected fetch: (input: RequestInfo, init?: RequestInit, auth?: boolean) => Promise<Response>;

	constructor(baseFetch: FetchType) {
		this.fetch = async (input: RequestInfo, init: RequestInit = {}, auth = true) => {
			const token = localStorage.getItem('sessionToken');
			const headers = new Headers(init.headers ?? {});

			if (auth && token) {
				headers.set('Authorization', `Bearer ${token}`);
			}

			const response = await baseFetch(input, {
				...init,
				headers,
			});

			if (auth && response.status === 401) {
				localStorage.removeItem('sessionToken');
				goto('/login');
			}

			return response;
		};
	}
}
