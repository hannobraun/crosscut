import { JSX } from "@bossley9/sjsx/jsx-runtime";
import * as gfm from "@deno/gfm";

export const dailyNotesPage = (dates: string[]) => {
    const entries = [];
    for (const date of dates) {
        entries.push(dailyNoteItem(date));
    }

    return page(
        "Daily Notes",
        <>
            <h2>Daily Notes</h2>
            {dailyNotesExplainer()}
            <ol class="m-8">{entries}</ol>
        </>,
    );
};

export const singleDailyNotePage = (
    date: string,
    md: string,
    dates: string[],
) => {
    let nameExplainer;
    if (new Date("2024-12-25") > new Date(date)) {
        nameExplainer = box(
            <p>
                This note was published {link(
                    "/daily/2024-12-25",
                    " before Crosscut was called Crosscut",
                )}! If it refers to "Caterpillar", that is the old name, just so
                you know.
            </p>,
        );
    }

    const html = gfm.render(md, {
        allowedTags: ["source"],
        allowedAttributes: { "source": ["src"] },
    });

    return page(
        `Daily Note - ${date}`,
        <>
            <h2>Daily Note - {date}</h2>
            {dailyNotesExplainer()}
            {nameExplainer}
            <div class="my-8">
                <nav>
                    {link("/daily", "< back to list")}
                </nav>
                <main class="prose">
                    {html}
                </main>
                <nav class="grid grid-cols-2">
                    {dailyNoteNavigation(date, dates)}
                </nav>
            </div>
        </>,
    );
};

const dailyNoteItem = (date: string) => {
    return (
        <li class="my-4 font-bold text-lg">
            {dailyNoteLink(date, date)}
        </li>
    );
};

const dailyNoteNavigation = (date: string, dates: string[]) => {
    const index = dates.findIndex((element) => element == date);

    const prev = dates[index + 1];
    const next = dates[index - 1];

    return (
        <>
            {prev && (
                <span class="col-1 justify-self-start">
                    {dailyNoteLink(prev, "<< previous note")}
                </span>
            )}
            {next && (
                <span class="col-2 justify-self-end">
                    {dailyNoteLink(next, "next note >>")}
                </span>
            )}
        </>
    );
};

const dailyNoteLink = (date: string, label: string) => {
    const url = `/daily/${date}`;
    return link(url, label);
};

const dailyNotesExplainer = () => {
    return box(
        <p class="prose">
            Hey, I'm Hanno! These are my daily notes on{" "}
            {link("https://github.com/hannobraun/crosscut", "Crosscut")}, the
            programming language I'm creating. If you have any questions,
            comments, or feedback, please {email_link("get in touch")}!
        </p>,
    );
};

const box = (content: JSX.Element, options?: { highlight: boolean }) => {
    let bgColor;
    if (options && options.highlight) {
        bgColor = "bg-yellow-200";
    } else {
        bgColor = "bg-slate-200";
    }

    const boxClass = `m-4 p-4 rounded font-sm ${bgColor}`;

    return <div class={boxClass}>{content}</div>;
};

const email_link = (
    text: string,
    extra?: { subject: string; body: string },
) => {
    const subject = extra && extra.subject || "";
    const body = extra && extra.body || "";

    const url =
        `mailto:Hanno%20Braun%20%3Chello%40hannobraun.com%3E?subject=${subject}&body=${body}`;

    return link(
        url,
        text,
    );
};

const link = (url: string, label: string) => {
    return (
        <a href={url} class="text-blue-700 underline font-bold">
            {label}
        </a>
    );
};

const page = (title: string, content: JSX.Element) => {
    return (
        <>
            {"<!doctype html>"}
            <html lang="en">
                <head>
                    <title>{title} - Crosscut</title>

                    <meta charSet="UTF-8" />
                    <meta
                        name="viewport"
                        content="width=device-width, initial-scale=1"
                    />

                    <link href="/style.css" rel="stylesheet" />
                </head>
                <body class="max-w-xl mx-auto p-2">
                    <header>
                        <a href="/">
                            <h1>Crosscut</h1>
                        </a>
                    </header>
                    <main>
                        {content}
                    </main>

                    <hr class="w-1/2 mx-auto my-16" />

                    <footer class="max-w-fit mx-auto text-sm">
                        <p class="max-w-fit mx-auto italic">A website by</p>
                        <address>
                            <div>
                                Hanno Braun<br />
                                Untere Pfarrgasse 19<br />
                                64720 Michelstadt<br />
                                Germany<br />
                            </div>
                            <div class="my-4">
                                📧{" "}
                                <a href="mailto:hello@hannobraun.com">
                                    hello@hannobraun.com
                                </a>
                            </div>
                        </address>
                    </footer>
                </body>
            </html>
        </>
    );
};
