===description===
FALSE POSITIVE reproducer. Valid PHP: `bzcompress`/`bzdecompress` are ext-bz2 built-in functions (a required extension); stubs are missing.
mir 0.42.0 currently emits (the bug): UndefinedFunction@4:9-4:26 (bzcompress) + UndefinedFunction@5:11-5:27 (bzdecompress)
Expected: no issue. Remove ===ignore=== to activate once fixed.
===config===
php_version=8.4
===file===
<?php
function pack_(string $data): void {
    // FP expected: UndefinedFunction bzcompress / bzdecompress (ext-bz2 stubs missing)
    $c = bzcompress($data);
    $d = bzdecompress('test');
    echo $c;
    echo $d;
}
===expect===
