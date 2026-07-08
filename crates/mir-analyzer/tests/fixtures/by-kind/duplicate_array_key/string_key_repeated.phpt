===description===
A repeated string key in an array literal silently overwrites the earlier
entry at runtime — almost always a copy-paste mistake.
===file===
<?php
function test(): array {
    return ['a' => 1, 'b' => 2, 'a' => 3];
}
===expect===
DuplicateArrayKey@3:32-3:35: Array key 'a' is duplicated — the earlier entry is silently overwritten
