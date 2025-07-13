import { redirect } from "@sveltejs/kit";

export const load = () => {
	// TODO: only redirect if not logged in.
	throw redirect(307, '/login');
};
