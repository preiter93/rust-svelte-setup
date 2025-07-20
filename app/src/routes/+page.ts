import { UserService } from "$lib/user/service";
import { redirect } from "@sveltejs/kit";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ fetch }) => {
	const token = localStorage.getItem('sessionToken');
	if (token === null || token === '') {
		throw redirect(307, '/login');
	}

	const user = await new UserService(fetch).getCurrentUser();

	return { user }
};
