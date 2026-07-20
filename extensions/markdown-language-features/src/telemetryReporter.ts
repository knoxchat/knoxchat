/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

export interface TelemetryReporter {
	dispose(): void;
	sendTelemetryEvent(eventName: string, properties?: {
		[key: string]: string;
	}): void;
}

class NoopTelemetryReporter implements TelemetryReporter {
	sendTelemetryEvent(): void { }
	dispose(): void { }
}

export function loadDefaultTelemetryReporter(): TelemetryReporter {
	return new NoopTelemetryReporter();
}
