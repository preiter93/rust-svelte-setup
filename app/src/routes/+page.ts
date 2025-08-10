import { UserService } from "$lib/user/service";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ fetch }) => {
	const user = await new UserService(fetch).getCurrentUser();
	return { user };
};
