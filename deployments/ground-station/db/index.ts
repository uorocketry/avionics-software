import { createClient } from "@libsql/client";
import { parseSqlStatements } from "./sql";

const DATABASE_URL = process.env.DATABASE_URL || "http://0.0.0.0:8080";

// Function to wait for database to be connectable
async function waitForDatabase(url: string, maxRetries = 30, baseDelay = 1000): Promise<void> {
  console.log(`Waiting for database to be available at ${url}...`);

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      const testClient = createClient({ url });
      // Try to execute a simple query to test connection
      await testClient.execute("SELECT 1");
      console.log(`Database is available after ${attempt} attempts`);
      return;
    } catch (error) {
      if (attempt === maxRetries) {
        console.error(`Failed to connect to database after ${maxRetries} attempts`);
        throw error;
      }

      const delay = baseDelay * Math.pow(2, attempt - 1);
      console.log(`Attempt ${attempt}/${maxRetries}: Database not ready, retrying in ${delay}ms...`);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
}

// Wait for database before proceeding
await waitForDatabase(DATABASE_URL);

const client = createClient({ url: DATABASE_URL });

const seed = await Bun.file(new URL("./seed.sql", import.meta.url)).text();
const statements = parseSqlStatements(seed);

let nErrors = 0;
for (const statement of statements) {
  try {
    await client.execute(statement);
    const firstLine = statement.split("\n")[0];
    console.log(`${firstLine}  ...OK`);
  } catch (e) {
    nErrors++;
    console.error("Statement failed:");
    console.log("--------------------------------");
    console.log(statement);
    console.log("--------------------------------");
    console.error(e);
    console.log("--------------------------------");
  }
}

if (nErrors > 0) {
  console.error(`Failed to execute ${nErrors} statements`);
  process.exit(1);
}

console.log(`Successfully executed ${statements.length} statements`);
