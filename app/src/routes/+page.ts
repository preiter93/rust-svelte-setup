import { UserService } from "$lib/user/service";
import { redirect } from "@sveltejs/kit";
import { HttpError } from "$lib/errors";
import type { PageLoad } from "./$types";
import type { User } from "$lib/protos/user/proto/User";

export const load: PageLoad = async ({ fetch }) => {
	const token = localStorage.getItem('sessionToken');
	if (token === null || token === '') {
		throw redirect(307, '/login');
	}

	const user = await new UserService(fetch).getCurrentUser();

	return { user }
};
