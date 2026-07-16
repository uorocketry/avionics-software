import { createClient, type Client } from "@libsql/client";

const LIBSQL_URL = process.env.DB_URL || "http://localhost:5001"; // Prefer Traefik DNS in deployment; override with DB_URL
let client: Client | null = null;

/**
 * Returns a singleton instance of the LibSQL database client.
 * Initializes the client on the first call.
 */
export function getDbClient(): Client {
	if (!client) {
		client = createClient({
			url: LIBSQL_URL,
		});
		console.log("LibSQL client initialized."); // More generic message now
	}
	return client;
}

// Optional: Add a function to close the client if needed during app shutdown
export function closeDbClient(): void {
	if (client) {
		client.close();
		client = null;
		console.log("LibSQL client closed.");
	}
}