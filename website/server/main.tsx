import * as http from "@std/http";

Deno.serve(async (request) => {
    const response = await servePage(request);

    // Not awaiting the request tracking, so it doesn't block the response.
    //
    // Currently disabled, because analyzing the result was more of a bother
    // than I was ready for.
    // trackRequest(request);

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

// Currently disabled. Leaving it in, in case I change my mind.
// async function trackRequest(request: Request) {
//     try {
//         const data = {
//             timestamp: new Date().toISOString(),
//             id: crypto.randomUUID(),
//             region: Deno.env.get("DENO_REGION"),
//             deploymentId: Deno.env.get("DENO_DEPLOYMENT_ID"),
//             method: request.method,
//             url: request.url,
//             userAgent: request.headers.get("user-agent"),
//             referrer: request.headers.get("referer"),
//         };

//         const kv = await Deno.openKv();
//         await kv.set(["analytics", data.timestamp, data.id], data);
//     } catch (error) {
//         console.error("Failed to track request:", error);
//     }
// }
