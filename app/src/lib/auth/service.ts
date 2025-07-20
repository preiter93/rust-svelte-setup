import type { CreateSessionReq } from "$lib/protos/auth/proto/CreateSessionReq";
import type { CreateSessionResp } from "$lib/protos/auth/proto/CreateSessionResp";
import { Google } from "arctic";
// import { GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET } from "$env/static/private";
const serverUrl = 'http://0.0.0.0:3000';

const GOOGLE_CLIENT_ID = "948143150752-gl142ug390d0im4fk34pj97k9o5amsl9.apps.googleusercontent.com"
const GOOGLE_CLIENT_SECRET = "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"

export const google = new Google(GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, "http://localhost:5173/login/google/callback");

type FetchType = (input: RequestInfo, init?: RequestInit) => Promise<Response>;
export class AuthService {
	constructor(fetch: FetchType) {
		this.fetch = fetch;
	}

	fetch: FetchType;

	async createSession(user_id: string | undefined): Promise<CreateSessionResp> {
		const request: CreateSessionReq = {
			user_id: user_id,
		};
		const response = await this.fetch(`${serverUrl}/session`, {
			method: "POST",
			body: JSON.stringify(request),
			headers: { 'Content-Type': 'application/json' },
		});
		if (!response.ok) {
			throw new Error(`failed to create session: ${response.statusText}`)
		}
		const data: CreateSessionResp = await response.json();
		return data;
	}
}
//
// 	async loginGoogle(): Promise<ListProgramsResp> {
// 		const response = await this.fetch(`${serverUrl}/programs`);
// 		if (!response.ok) {
// 			throw new Error(`failed to list programs: ${response.statusText}`);
// 		}
// 		const data: ListProgramsResp = await response.json();
// 		return data;
// 	}
//
// 	async getProgram(id: string): Promise<GetProgramResp> {
// 		const response = await this.fetch(`${serverUrl}/programs/${id}`);
// 		if (!response.ok) {
// 			throw new Error(`failed to get program: ${response.statusText}`);
// 		}
// 		const data: GetProgramResp = await response.json();
// 		return data;
// 	}
//
// 	async updateDay(id: string, done: boolean): Promise<void> {
// 		const payload = { done };
// 		const response = await this.fetch(`${serverUrl}/days/${id}`, {
// 			method: 'PUT',
// 			headers: { 'Content-Type': 'application/json' },
// 			body: JSON.stringify(payload),
// 		});
//
// 		if (!response.ok) {
// 			throw new Error(`Failed to update day: ${response.statusText}`);
// 		}
// 	}
//
// 	async listRuns(): Promise<Runs> {
// 		const response = await this.fetch(`${serverUrl}/runs`);
// 		if (!response.ok) {
// 			throw new Error(`failed to list runs: ${response.statusText}`);
// 		}
// 		const data: Runs = await response.json();
// 		return data;
// 	}
//
// 	async syncRuns(): Promise<void> {
// 		const response = await this.fetch(`${serverUrl}/polar/runs:sync`, {
// 			method: 'POST',
// 			headers: { 'Content-Type': 'application/json' },
// 		});
//
// 		if (!response.ok) {
// 			throw new Error(`Failed to sync runs: ${response.statusText}`);
// 		}
// 	}
//
// 	async listSamples(id: string): Promise<Samples> {
// 		const response = await this.fetch(`${serverUrl}/runs/${id}/samples`);
// 		if (!response.ok) {
// 			throw new Error(`failed to list samples: ${response.statusText}`);
// 		}
// 		const data: Samples = await response.json();
// 		return data;
// 	}
// }
//
// export interface Run {
// 	id: string,
// 	distance: number,
// 	hear_rate_average: number,
// 	hear_rate_max: number,
// 	speed_average: number,
// 	speed_max: number,
// 	start_time: string,
// 	samples: Sample[],
// }
//
// export interface Runs {
// 	runs: Run[],
// }
//
// export interface Sample {
// 	run_id: string,
// 	speed: number,
// 	heart_rate: number,
// 	offset: number,
// }
//
// // export function avgSpeedFromSamples(samples: Sample[]): number {
// // 	if (samples.length === 0) return 0;
// //
// // 	const total = samples.reduce((sum, sample) => sum + sample.speed, 0);
// // 	return total / samples.length;
// // }
//
// export interface Samples {
// 	samples: Sample[],
// }
//
// export type ListProgramsResp = string[];
//
// export type GetProgramResp = Program;
//
// export interface Program {
// 	id: string;
// 	name: string;
// 	createdAt: string | null;
// 	weeks: Week[];
// }
//
// export interface Week {
// 	id: string;
// 	program_id: string;
// 	number: number;
// 	days: Day[];
// }
//
// export interface Day {
// 	id: string;
// 	week_id: string;
// 	number: number;
// 	workout: Workout;
// 	done: boolean;
// }
//
// export type Workout =
// 	| { kind: 'easy'; distance: number, pace: number }
// 	| { kind: 'tempo'; distance: number, pace: number }
// 	| { kind: 'interval'; distance: number, pace: number, interval: number }
// 	| { kind: 'rest' };
// // | { type: 'interval'; repeats: number; distance: number; pace: string }
// // | { type: 'crosstrain'; duration: number }
// //
// // | { type: 'rest' }
// // | { type: 'race'; distance: number };
//
