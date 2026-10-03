===description===
Includes nested in conditionals and function bodies are still discovered
===config===
file_extensions=module,inc
include_seed=a.module
===file:a.module===
<?php
function a_fn(bool $f): int {
    if ($f) {
        require_once __DIR__ . '/b.inc';
    }
    return b_fn();
}
===file:b.inc===
<?php
function b_fn(): int { return 1; }
===expect===
