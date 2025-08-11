import { goto } from '$app/navigation';

export type FetchType = typeof fetch;

export class BaseService {
	protected fetch: (input: RequestInfo, init?: RequestInit) => Promise<Response>;

	constructor(baseFetch: FetchType) {
		this.fetch = async (input: RequestInfo, init: RequestInit = {}) => {
			const headers = new Headers(init.headers ?? {});

			const response = await baseFetch(input, {
				...init,
				headers,
				credentials: 'include',
			});

			if (response.status === 401) {
				goto('/login');
			}

			return response;
		};
	}
}
