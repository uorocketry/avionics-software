// This file will handle loading existing commands and processing the dispatch form action.
import { getDbClient } from '$lib/server/db';
import { fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

// Load existing commands
export const load: PageServerLoad = async () => {
	console.log('Loading commands for /commands page...');
	const db = getDbClient();
	try {
		const result = await db.execute({
			sql: 'SELECT id, command_type, parameters, status, created_at, queued_at, sent_at, attempts, error_message, source_service FROM OutgoingCommand ORDER BY created_at DESC LIMIT 100',
			args: []
		});
		console.log(`Loaded ${result.rows.length} commands.`);
		// Convert row data if necessary (e.g., timestamps)
		const commands = result.rows.map(row => ({
			...row,
			// Ensure numeric fields are numbers, text is string, etc.
			id: Number(row.id),
			created_at: Number(row.created_at),
			queued_at: row.queued_at ? Number(row.queued_at) : null,
			sent_at: row.sent_at ? Number(row.sent_at) : null,
			attempts: Number(row.attempts),
			// Keep text fields as is (assuming they come correctly typed from libsql client)
			command_type: String(row.command_type),
			parameters: row.parameters as string | null, // Type assertion might be needed depending on client
			status: String(row.status),
			error_message: row.error_message as string | null,
			source_service: String(row.source_service)
		}));
		return { commands };
	} catch (e: any) {
		console.error('Error loading commands:', e);
		// Return empty array or an error state
		return { commands: [], error: 'Failed to load commands.' };
	}
};

// Handle form submission for dispatching new commands and deleting existing ones
export const actions: Actions = {
	dispatch: async ({ request }) => {
		const formData = await request.formData();
		const command_type = formData.get('command_type') as string;

		let parameters: string | null = null;
		const params: Record<string, any> = {};

		try {
			switch (command_type) {
				case 'DeployDrogue':
				case 'DeployMain':
					// These commands might not need user parameters, set defaults if needed
					params.val = true; // Example default
					break;
				case 'PowerDown':
					const board = formData.get('param_board') as string;
					if (!board) throw new Error('Board parameter is required for PowerDown.');
					params.board = board;
					break;
				case 'PowerUpCamera':
					// no parameters
					break;
				case 'PowerDownCamera':
					// no parameters
					break;
				case 'RadioRateChange':
					const rate = formData.get('param_rate') as string;
					if (!rate) throw new Error('Rate parameter is required for RadioRateChange.');
					params.rate = rate;
					break;
				case 'Ping':
					// No parameters needed
					break;
				default:
					return fail(400, { error: `Unknown command type: ${command_type}` });
			}

			// Only serialize parameters if needed
			if (Object.keys(params).length > 0) {
				parameters = JSON.stringify(params);
			}

			const db = getDbClient();
			const now = Math.floor(Date.now() / 1000);
			const source_service = 'rgs-web-commands';

			console.log(`Dispatching command: ${command_type} with params: ${parameters}`);

			await db.execute({
				sql: 'INSERT INTO OutgoingCommand (command_type, parameters, status, created_at, attempts, source_service) VALUES (?, ?, ?, ?, ?, ?)',
				args: [command_type, parameters, 'Pending', now, 0, source_service]
			});

			console.log('Command inserted successfully.');
			return { success: true, message: `Command '${command_type}' dispatched.` };

		} catch (e: any) {
			console.error('Error dispatching command:', e);
			return fail(500, { error: e.message || 'Failed to dispatch command.' });
		}
	},

	// Action to delete a single command
	deleteSingle: async ({ request }) => {
		const formData = await request.formData();
		const idValue = formData.get('id');

		if (!idValue || typeof idValue !== 'string') {
			return fail(400, { error: 'Invalid or missing command ID for deletion.' });
		}
		const id = idValue as string;

		const db = getDbClient();
		try {
			console.log(`Deleting command with ID: ${id}`);
			const result = await db.execute({
				sql: 'DELETE FROM OutgoingCommand WHERE id = ?',
				args: [id]
			});

			if (result.rowsAffected === 0) {
				console.warn(`Command with ID ${id} not found for deletion.`);
				return fail(404, { error: `Command with ID ${id} not found.` });
			}

			console.log(`Command ${id} deleted successfully.`);
			return { success: true, message: `Command ${id} deleted.` };
		} catch (e: any) {
			console.error(`Error deleting command ${id}:`, e);
			return fail(500, { error: e.message || `Failed to delete command ${id}.` });
		}
	},

	// Action to delete all commands
	deleteAll: async () => {
		const db = getDbClient();
		try {
			console.log('Deleting all commands...');
			const result = await db.execute({
				sql: 'DELETE FROM OutgoingCommand',
				args: []
			});
			console.log(`${result.rowsAffected} commands deleted.`);
			return { success: true, message: `All (${result.rowsAffected}) commands deleted.` };
		} catch (e: any) {
			console.error('Error deleting all commands:', e);
			return fail(500, { error: e.message || 'Failed to delete all commands.' });
		}
	}
};