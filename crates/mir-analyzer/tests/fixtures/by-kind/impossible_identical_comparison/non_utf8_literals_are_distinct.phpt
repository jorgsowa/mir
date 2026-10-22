===description===
Distinct non-UTF-8 string literals are never identical, and the message shows their bytes.
===file===
<?php
function f(): void {
    $a = "\xff";
    if ($a === "\xfe") {
    }
    if ($a === "\xff") {
    }
}
===expect===
ImpossibleIdenticalComparison@4:8-4:21: '===' between '"\xff"' and '"\xfe"' is always false — these types can never be identical
