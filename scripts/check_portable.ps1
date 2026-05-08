# PowerShell script to scan for absolute path leaks after relocation.
# Run this from the project root.

$OldRoot = "C:\Users\Home Office_PC"
echo "`n🔍 Checking for path leaks in Tadpole OS codebase...`n"

# Files to ignore (node_modules, target, etc.)
$Ignore = @(".git", "node_modules", "target", ".agent", ".tmp", "dist", "sidecar_runtime.log")

Get-ChildItem -Recurse -File -Exclude $Ignore | ForEach-Object {
    $FilePath = $_.FullName
    $RelativePath = Resolve-Path $FilePath -Relative
    
    # Try to read the file content
    try {
        $Content = Get-Content $FilePath -ErrorAction SilentlyContinue
        if ($null -ne $Content) {
            $Match = $Content | Select-String -Pattern [regex]::Escape($OldRoot)
            if ($null -ne $Match) {
                echo "❌ LEAK FOUND: $RelativePath"
                $Match | ForEach-Object { echo "   Line $($_.LineNumber): $($_.Line.Trim())" }
            }
        }
    } catch {
        # Skip binary files or locked files
    }
}

echo "`n✅ Path scan complete. If no 'LEAK FOUND' messages appeared above, the codebase is 100% portable.`n"
