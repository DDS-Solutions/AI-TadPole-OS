import fs from 'fs'
import path from 'path'

const binDir = path.resolve('node_modules/.bin')
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true })
}

const shims = [
  { name: 'vite', relFromBin: '../vite/bin/vite.js' },
  { name: 'vitest', relFromBin: '../vitest/vitest.mjs' },
  { name: 'eslint', relFromBin: '../eslint/bin/eslint.js' },
  { name: 'tsc', relFromBin: '../typescript/bin/tsc' },
  { name: 'vitepress', relFromBin: '../vitepress/bin/vitepress.js' }
]

for (const { name, relFromBin } of shims) {
  const shContent = `#!/bin/sh
basedir=$(dirname "$(echo "$0" | sed -e 's,\\\\,/,g')")
exec node "$basedir/${relFromBin}" "$@"
`
  const cmdContent = `@IF EXIST "%~dp0\\node.exe" (
  "%~dp0\\node.exe"  "%~dp0\\${relFromBin.replace(/\//g, '\\')}" %*
) ELSE (
  @SETLOCAL
  @SET PATHEXT=%PATHEXT:;.JS;=;%
  node  "%~dp0\\${relFromBin.replace(/\//g, '\\')}" %*
)
`
  const ps1Content = `#!/usr/bin/env pwsh
$basedir=Split-Path $MyInvocation.MyCommand.Definition -Parent
$exe="node"
& "$exe"  "$basedir/${relFromBin}" $args
exit $LASTEXITCODE
`

  fs.writeFileSync(path.join(binDir, name), shContent, { mode: 0o755 })
  fs.writeFileSync(path.join(binDir, `${name}.cmd`), cmdContent, { mode: 0o755 })
  fs.writeFileSync(path.join(binDir, `${name}.ps1`), ps1Content, { mode: 0o755 })
  console.log(`Generated shim for ${name}`)
}
