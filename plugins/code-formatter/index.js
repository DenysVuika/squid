function execute(context, input) {
    try {
        context.log(`Formatting ${input.language} code`);
        
        const indentSize = input.indent_size || 2;
        const indent = ' '.repeat(indentSize);
        let formatted = input.code;
        let changes = 0;
        
        if (input.language === 'json') {
            // Format JSON
            try {
                const parsed = JSON.parse(input.code);
                formatted = JSON.stringify(parsed, null, indentSize);
                changes = countDifferences(input.code, formatted);
            } catch (e) {
                context.log(`JSON parsing failed: ${e.message}`);
                return {
                    formatted: input.code,
                    changes: 0,
                    language: input.language,
                    error: 'Invalid JSON'
                };
            }
        } else if (input.language === 'javascript') {
            // Basic JavaScript formatting (simplified)
            const lines = input.code.split('\n');
            let depth = 0;
            const formattedLines = [];
            
            for (const line of lines) {
                const trimmed = line.trim();
                
                // Decrease depth for closing braces/brackets
                if (trimmed.match(/^[}\]]/)) {
                    depth = Math.max(0, depth - 1);
                }
                
                // Add indented line
                if (trimmed) {
                    formattedLines.push(indent.repeat(depth) + trimmed);
                } else {
                    formattedLines.push('');
                }
                
                // Increase depth for opening braces/brackets
                if (trimmed.match(/[{[]$/)) {
                    depth++;
                }
                // Decrease depth if line has both opening and closing
                if (trimmed.match(/^[}\]]/)) {
                    // Already handled above
                }
            }
            
            formatted = formattedLines.join('\n');
            changes = countDifferences(input.code, formatted);
        } else if (input.language === 'markdown') {
            // Basic markdown formatting
            const lines = input.code.split('\n');
            const formattedLines = [];
            
            for (let i = 0; i < lines.length; i++) {
                let line = lines[i];
                
                // Remove trailing whitespace
                line = line.replace(/\s+$/, '');
                
                // Ensure proper spacing around headers
                if (line.match(/^#+\s/)) {
                    if (i > 0 && formattedLines[formattedLines.length - 1].trim() !== '') {
                        formattedLines.push('');
                    }
                }
                
                formattedLines.push(line);
            }
            
            formatted = formattedLines.join('\n');
            changes = countDifferences(input.code, formatted);
        }
        
        context.log(`Formatting complete: ${changes} changes`);
        
        return {
            formatted,
            changes,
            language: input.language
        };
    } catch (error) {
        return {
            formatted: input.code,
            changes: 0,
            language: input.language,
            error: error.message || 'Unknown error'
        };
    }
}

function countDifferences(original, formatted) {
    const originalLines = original.split('\n');
    const formattedLines = formatted.split('\n');
    
    let diff = 0;
    const maxLen = Math.max(originalLines.length, formattedLines.length);
    
    for (let i = 0; i < maxLen; i++) {
        if (originalLines[i] !== formattedLines[i]) {
            diff++;
        }
    }
    
    return diff;
}

globalThis.execute = execute;
