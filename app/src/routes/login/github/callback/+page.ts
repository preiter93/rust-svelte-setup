import type { PageLoad } from "./$types";

export const load: PageLoad = ({ url }) => {
	const code = url.searchParams.get('code');
	const state = url.searchParams.get('state');

	return { code, state };
};
