function execute(context, input) {
    try {
        context.log(`Linting markdown file: ${input.path}`);
        
        // Read the file
        const content = context.readFile(input.path);
        const maxLineLength = input.max_line_length || 120;
        
        const issues = [];
        const lines = content.split('\n');
        
        // Check line length
        lines.forEach((line, index) => {
            if (line.length > maxLineLength) {
                issues.push(
                    `Line ${index + 1}: Exceeds maximum length (${line.length} > ${maxLineLength})`
                );
            }
        });
        
        // Check for trailing whitespace
        lines.forEach((line, index) => {
            if (line.match(/\s+$/)) {
                issues.push(`Line ${index + 1}: Contains trailing whitespace`);
            }
        });
        
        // Check for multiple consecutive blank lines
        for (let i = 0; i < lines.length - 2; i++) {
            if (lines[i].trim() === '' && 
                lines[i + 1].trim() === '' && 
                lines[i + 2].trim() === '') {
                issues.push(`Line ${i + 1}: Multiple consecutive blank lines`);
            }
        }
        
        // Calculate stats
        const headings = content.match(/^#+\s+/gm) || [];
        const codeBlocks = content.match(/```/g) || [];
        
        const stats = {
            lines: lines.length,
            headings: headings.length,
            code_blocks: codeBlocks.length / 2  // Divide by 2 since each block has opening and closing
        };
        
        context.log(`Found ${issues.length} issues in ${lines.length} lines`);
        
        return { issues, stats };
    } catch (error) {
        return {
            error: error.message || 'Unknown error',
            issues: [],
            stats: { lines: 0, headings: 0, code_blocks: 0 }
        };
    }
}

globalThis.execute = execute;
