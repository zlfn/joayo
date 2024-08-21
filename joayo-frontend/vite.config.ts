import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig, searchForWorkspaceRoot } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		fs: {
			allow: [
				// This is required for run VITE server in bazel_bin folder.
				searchForWorkspaceRoot(process.cwd()),
			],
		},
	},
});
