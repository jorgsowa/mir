===description===
An explicit int key that collides with a preceding auto-incremented
positional key is the same silent-overwrite bug, not just two identical
explicit keys.
===file===
<?php
function test(): array {
    return ['b', 0 => 'a'];
}
function no_collision(): array {
    return [1 => 'a', 'b'];
}
===expect===
DuplicateArrayKey@3:17-3:18: Array key 0 is duplicated — the earlier entry is silently overwritten
