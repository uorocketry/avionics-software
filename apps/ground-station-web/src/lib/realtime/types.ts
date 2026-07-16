export type ConnectionStatus = 'connecting' | 'live' | 'stale' | 'offline' | 'error';

export type TimedValue<T> = {
	value: T | null;
	updatedAt: number | null;
};

export type FlightTransition = {
	id: number;
	kind: 'State' | 'Event';
	value: string;
	timestamp: number | null;
};

export type TrajectoryPoint = {
	timestamp: number;
	lat: number;
	lon: number;
};

export type OverviewSnapshot = {
	generatedAt: number;
	flight: {
		state: FlightTransition | null;
		event: FlightTransition | null;
		altitudeM: TimedValue<number>;
		gnssAltitudeM: TimedValue<number>;
		verticalVelocityMps: TimedValue<number>;
		accelerationMps2: TimedValue<number>;
		position: TimedValue<{ lat: number; lon: number }>;
		satellites: TimedValue<number>;
	};
	radio: {
		updatedAt: number | null;
		rssiDbm: number | null;
		framesPerSecond: number;
		packetLossPercent: number;
		totalFrames: number;
	};
	services: {
		updatedAt: number;
		running: number;
		total: number;
		degraded: string[];
		available: boolean;
	};
	recentTransitions: FlightTransition[];
	trajectory: TrajectoryPoint[];
};
