const fs = require('fs');
const path = require('path');
const indexPath = path.join(__dirname, '..', 'nuxt-app', '.output', 'public', 'index.html');
if (fs.existsSync(indexPath)) {
  let html = fs.readFileSync(indexPath, 'utf-8');
  html = html.replace(/href="\/_nuxt\//g, 'href="./_nuxt/');
  html = html.replace(/src="\/_nuxt\//g, 'src="./_nuxt/');
  fs.writeFileSync(indexPath, html);
  console.log('Fixed asset paths for Tauri (file:// protocol)');
} else {
  console.log('index.html not found at', indexPath);
}
