===description===
Distinct non-UTF-8 string literals are never identical, and the message shows their bytes.
===file===
<?php
function f(): void {
    $a = "\xff";
    if ($a === "\xfe") {
//      ^^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '"\xff"' and '"\xfe"' is always false — these types can never be identical
    }
    if ($a === "\xff") {
    }
}
