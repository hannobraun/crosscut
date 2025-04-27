export const listDailyNotes = async () => {
    const dates = [];

    for await (const dirEntry of Deno.readDir("content/daily")) {
        const date = dirEntry.name.match(
            /^(\d{4}-\d{2}-\d{2}).md$/,
        );

        if (date) {
            dates.push(date[1]);
        }
    }

    dates.sort();
    dates.reverse();

    return dates;
};
