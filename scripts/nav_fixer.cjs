const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, 'src');

function getAllFiles(dir, fileList = []) {
  if (!fs.existsSync(dir)) return fileList;
  const files = fs.readdirSync(dir);
  files.forEach(file => {
    const filePath = path.join(dir, file);
    if (fs.statSync(filePath).isDirectory()) {
      getAllFiles(filePath, fileList);
    } else {
      fileList.push(filePath);
    }
  });
  return fileList;
}

const allFiles = getAllFiles(srcDir);

allFiles.forEach(filePath => {
  const ext = path.extname(filePath);
  if (ext !== '.ts' && ext !== '.tsx') return;

  let content = fs.readFileSync(filePath, 'utf8');
  const originalContent = content;

  // 1. Fix NavLink destructuring: { is_active } -> { isActive: is_active }
  // This satisfies react-router-dom while keeping the local variable snake_case
  content = content.replace(/\{(\s*)is_active(\s*)\}(\s*)=>(\s*)/g, '{ isActive: is_active } => ');
  
  // Also handle cases with parens Around the object
  content = content.replace(/\((\s*)\{(\s*)is_active(\s*)\}(\s*)\)(\s*)=>(\s*)/g, '({ isActive: is_active }) => ');

  // 2. Fix interfaces that were changed to is_active but should be isActive to match NavLinkRenderProps
  // Or keep them as is and we fix the call site, but usually they are typed to match the library
  content = content.replace(/\(props: \{(\s*)is_active: boolean(\s*)\}\)/g, '(props: { isActive: boolean })');

  // 3. Fix the definition of nav_item_class in Dashboard_Layout
  content = content.replace(/const nav_item_class = \(\{\s*is_active\s*\}\s*:\s*\{\s*is_active\s*:\s*boolean\s*\}\)\s*=>/g, 'const nav_item_class = ({ isActive: is_active }: { isActive: boolean }) =>');

  if (content !== originalContent) {
    fs.writeFileSync(filePath, content, 'utf8');
    console.log(`Updated Nav Casing: ${filePath}`);
  }
});
