import { redirect } from "@sveltejs/kit";

export const load = () => {
	const token = localStorage.getItem('sessionToken');
	if (token === null || token === '') {
		throw redirect(307, '/login');
	}
};
