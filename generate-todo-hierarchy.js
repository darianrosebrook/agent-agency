import fs from 'fs';

try {
  const data = JSON.parse(fs.readFileSync('files-with-counts.json', 'utf8'));

  // Build hierarchical structure
  const tree = {};

  data.forEach(item => {
    const parts = item.file.split('/');
    let current = tree;

    parts.forEach((part, index) => {
      if (!current[part]) {
        current[part] = {};
      }
      current = current[part];

      if (index === parts.length - 1) {
        current.__count = item.count;
        current.__isFile = true;
      }
    });
  });

  // Generate markdown with simplified structure
  function generateMarkdown(node, prefix = '', level = 1) {
    let md = '';

    // Get all entries and sort them (directories first, then files)
    const entries = Object.entries(node).filter(([key]) => key !== '__count' && key !== '__isFile');
    const dirs = entries.filter(([, value]) => !value.__isFile);
    const files = entries.filter(([, value]) => value.__isFile);

    // Process directories
    for (const [key, value] of dirs) {
      const fullPath = prefix ? `${prefix}/${key}` : key;
      const headingLevel = '#'.repeat(Math.min(level, 3)); // Max heading level 3
      md += `${headingLevel} ${fullPath}/\n\n`;
      md += generateMarkdown(value, fullPath, level + 1);
      md += '\n';
    }

    // Process files in this directory
    if (files.length > 0) {
      if (dirs.length > 0) {
        md += '## Files\n\n';
      }
      for (const [key, value] of files) {
        const fullPath = prefix ? `${prefix}/${key}` : key;
        md += `- \`${fullPath}\` - **${value.__count}** TODOs\n`;
      }
      md += '\n';
    }

    return md;
  }

  const totalIssues = data.reduce((sum, item) => sum + item.count, 0);
  const markdown = `# TODO Analysis by Directory Structure

**Files analyzed:** ${data.length}  
**Total TODO issues:** ${totalIssues}

${generateMarkdown(tree)}
`;

  fs.writeFileSync('todo-files-by-path.md', markdown);
  console.log('Created todo-files-by-path.md');

} catch (error) {
  console.error('Error:', error.message);
}
