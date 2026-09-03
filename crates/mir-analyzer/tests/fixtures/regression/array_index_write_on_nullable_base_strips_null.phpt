===description===
An array-index write (`$data['key'] = ...`) on a nullable-array base
auto-vivifies at runtime (PHP creates a fresh array, no warning) — the
base's type must lose its `null` possibility after the write, both for a
literal key and for push notation (`$data[] = ...`).
===file===
<?php
function withKey(?array $data): array {
    $data['files'] = 'x';
    $data['other'] = 'y';
    return $data;
}
function withPush(?array $data): array {
    $data[] = 'x';
    return $data;
}
===expect===
