function execute(context, input) {
    try {
        context.log(`Fetching URL: ${input.url}`);
        
        const timeout = input.timeout || 5000;
        
        // Fetch content using context API (handled by Rust with proper async)
        const data = context.httpGet(input.url, timeout);
        
        context.log(`Request completed for: ${input.url} (${data.length} bytes)`);
        
        return {
            success: true,
            data: data,
            url: input.url,
            size: data.length
        };
    } catch (error) {
        context.log(`Request failed: ${error.message}`);
        
        return {
            success: false,
            error: error.message || 'Unknown error',
            url: input.url
        };
    }
}

globalThis.execute = execute;
