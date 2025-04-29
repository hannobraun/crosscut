import * as http from "@std/http";

import * as content from "./content.ts";
import * as response from "./response.ts";
import { dailyNotesPage, singleDailyNotePage } from "./templates.tsx";

Deno.serve(async (request) => {
    const response = await servePage(request);

    return response;
});

async function servePage(request: Request) {
    const url = new URL(request.url);

    if (
        url.hostname == "crosscut.deno.dev" ||
        url.hostname == "capi.hannobraun.com" || url.hostname == "crosscut.cc"
    ) {
        const pathname = new URL(request.url).pathname;
        return Response.redirect(
            `https://www.crosscut.cc/${pathname}`,
            308,
        );
    }

    if (url.pathname == "/") {
        return Response.redirect(
            `${url.origin}/daily`,
            307,
        );
    }
    if (url.pathname == "/daily/") {
        return Response.redirect(
            `${url.origin}/daily`,
            307,
        );
    }

    if (url.pathname == "/daily") {
        const dates = await content.listDailyNotes();
        const page = dailyNotesPage(dates);
        return response.page(page);
    }

    const dailyDateWithSlash = url.pathname.match(
        /^\/daily\/(\d{4}-\d{2}-\d{2})\/$/,
    );
    if (dailyDateWithSlash && dailyDateWithSlash[1]) {
        return Response.redirect(
            `${url.origin}/daily/${dailyDateWithSlash[1]}`,
            307,
        );
    }

    const dailyDateWithNoSlash = url.pathname.match(
        /^\/daily\/(\d{4}-\d{2}-\d{2})$/,
    );
    if (dailyDateWithNoSlash && dailyDateWithNoSlash[1]) {
        const date = dailyDateWithNoSlash[1];
        const path = `content/daily/${date}.md`;
        const md = await Deno.readTextFile(path);

        const dates = await content.listDailyNotes();

        const page = singleDailyNotePage(date, md, dates);

        return response.page(page);
    }

    return http.serveDir(request, {
        fsRoot: "static",
    });
}
