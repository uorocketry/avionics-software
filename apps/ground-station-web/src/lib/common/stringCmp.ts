// https://github.com/aceakash/string-similarity

/**
 * Calculates the similarity between two strings.
 * @param first string to compare
 * @param second string to compare
 * @returns a number between 0 and 1, where 1 is a perfect match
 */

export function getStringDifference(first: string, second: string) {
	first = first.replace(/\s+/g, '');
	second = second.replace(/\s+/g, '');

	if (first === second) return 1; // identical or empty
	if (first.length < 2 || second.length < 2) return 0; // if either is a 0-letter or 1-letter string

	const firstBigrams = new Map();
	for (let i = 0; i < first.length - 1; i++) {
		const bigram = first.substring(i, i + 2);
		const count = firstBigrams.has(bigram) ? firstBigrams.get(bigram) + 1 : 1;

		firstBigrams.set(bigram, count);
	}

	let intersectionSize = 0;
	for (let i = 0; i < second.length - 1; i++) {
		const bigram = second.substring(i, i + 2);
		const count = firstBigrams.has(bigram) ? firstBigrams.get(bigram) : 0;

		if (count > 0) {
			firstBigrams.set(bigram, count - 1);
			intersectionSize++;
		}
	}

	return (2.0 * intersectionSize) / (first.length + second.length - 2);
}

/**
 * Helper function that maps all strings to a score
 * but uses shortcuts to avoid calling getStringDifference on all strings
 * @param mainString
 * @param targetStrings
 * @returns an array with the score of each string in the array
 */
export function getStringScores(baseString: string, stringLst: string[]): number[] {
	baseString = baseString.toLowerCase();
	// Calculate starts with score
	const scores: number[] = stringLst.map((curStr) => {
		curStr = curStr.toLocaleLowerCase().replace('-', ' ');
		if (curStr === baseString) {
			return 4;
		}
		// Starting or ending with gets priority over including
		if (curStr.startsWith(baseString)) {
			return 3;
		}

		// Starting or ending with gets priority over including
		if (curStr.endsWith(baseString)) {
			return 2;
		}

		if (curStr.includes(baseString)) {
			return 1;
		}
		return getStringDifference(baseString, curStr);
	});

	return scores;
}

export function sum(a: number, b: number) {
	return a + b;
}
