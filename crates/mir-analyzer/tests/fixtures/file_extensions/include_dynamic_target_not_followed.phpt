===description===
Targets that depend on a runtime value are not resolved statically
===config===
file_extensions=module,inc
include_seed=a.module
===file:a.module===
<?php
function a_fn(string $name): int {
    include __DIR__ . '/' . $name . '.inc';
    return b_fn();
}
===file:b.inc===
<?php
function b_fn(): int { return 1; }
===expect===
a.module: MixedReturnStatement@4:4-4:18: Cannot return a mixed type from function with declared return type 'int'
a.module: UndefinedFunction@4:11-4:17: Function b_fn() is not defined
