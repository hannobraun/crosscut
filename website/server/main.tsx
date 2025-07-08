import * as http from "@std/http";

Deno.serve(async (request) => {
    const response = await servePage(request);
    return response;
});

function servePage(request: Request) {
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

    return http.serveDir(request, {
        fsRoot: "static",
    });
}
