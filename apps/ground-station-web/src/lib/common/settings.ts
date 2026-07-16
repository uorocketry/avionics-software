import { writable, type Writable } from 'svelte/store';
import { persistentStore } from './persistentStorage';

// Settings type definitions

type ValueT =
	| {
			valueDescription: 'string';
			value: Writable<string>;
			placeholder?: string;
	  }
	| {
			valueDescription: 'number';
			value: Writable<number>;
			min?: number;
			max?: number;
			step?: number;
	  }
	| {
			valueDescription: 'boolean';
			value: Writable<boolean>;
	  }
	| {
			valueDescription: 'array';
			value: Writable<string[]>;
	  }
	| {
			valueDescription: 'kv';
			value: Writable<{ [key: string]: string }>;
	  }
	| {
			valueDescription: 'enum';
			options: string[];
			value: Writable<string>;
	  };

type Setting = { name: string; description?: string } & ValueT;

type SettingsGroup = {
	name: string;
	settings: Setting[];
};

// Settings

const uiSettings = {
	name: 'ui',
	settings: [
		{
			name: 'theme',
			description: 'Select the application theme',
			valueDescription: 'enum',
			options: ['white', 'g100'],
			value: persistentStore('ui.theme', 'white')
		}
	]
} satisfies SettingsGroup;

const notificationSettings = {
	name: 'notifications',
	settings: [
		{
			name: 'consoleNotifications',
			description: 'Enable on-screen console notifications',
			valueDescription: 'boolean',
			value: persistentStore('notifications.consoleNotifications', false)
		}
	]
} satisfies SettingsGroup;

const flightSettings = {
	name: 'flight',
	settings: [
		// Altitude page
		{
			name: 'altitudeMinutes',
			valueDescription: 'number',
			description: 'Minutes to look back for altitude',
			value: persistentStore('flight.altitudeMinutes', 15),
			min: 1,
			max: 1440,
			step: 1
		},
		{
			name: 'altitudeQnhKpa',
			valueDescription: 'number',
			description: 'QNH (kPa) for barometric altitude',
			value: persistentStore('flight.altitudeQnhKpa', 100),
			min: 80,
			max: 110,
			step: 0.1
		},
		// IMU page
		{
			name: 'imuMinutes',
			valueDescription: 'number',
			description: 'Minutes to look back for IMU',
			value: persistentStore('flight.imuMinutes', 10),
			min: 1,
			max: 1440,
			step: 1
		},
		// Trajectory page
		{
			name: 'trajMinutes',
			valueDescription: 'number',
			description: 'Minutes to look back for trajectory',
			value: persistentStore('flight.trajMinutes', 240),
			min: 1,
			max: 1440,
			step: 1
		},
		{
			name: 'trajMaxPoints',
			valueDescription: 'number',
			description: 'Max points on map path',
			value: persistentStore('flight.trajMaxPoints', 3000),
			min: 100,
			max: 20000,
			step: 100
		},
		{
			name: 'trajRadiusKm',
			valueDescription: 'number',
			description: 'Radius around center (km)',
			value: persistentStore('flight.trajRadiusKm', 50),
			min: 1,
			max: 500,
			step: 1
		}
	]
} satisfies SettingsGroup;

export const settings = [uiSettings, notificationSettings, flightSettings];

export const findSetting = (name: string): Setting | undefined => {
	const groupName = name.split('.')[0];
	const group = settings.find((group) => group.name === groupName);
	if (group) {
		// Find setting in group
		const settingName = name.split('.')[1];
		const setting = group.settings.find((setting) => setting.name === settingName);
		if (setting) {
			return setting;
		}
	}

	return undefined;
};
