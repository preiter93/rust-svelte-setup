import type { PageLoad } from "./$types";

export const load: PageLoad = ({ params, url }) => {
	const provider = params.provider;
	const code = url.searchParams.get('code');
	const state = url.searchParams.get('state');

	return { provider, code, state };
};
