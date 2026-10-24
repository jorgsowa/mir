===description===
String literals that are not valid UTF-8 keep their bytes: `"\xff"` and
`"\xfe"` are different values in narrowing, `match` / `switch` duplicate and
coverage checks, and `"\xff" . "a"` equals `"\xffa"`.
===file===
<?php
function f(): void {
    $b = rand(0, 1) ? "\xff" : "\xfe";
    if ($b === "\xff") { echo "z"; } elseif ($b === "\xfe") { echo "w"; }
    echo match ($b) { "\xff" => 1, "\xfe" => 2 };
    switch ($b) { case "\xff": break; case "\xfe": break; }
    $c = "\xff" . "a";
    if ($c === "\xffa") { echo 1; }
    $k = ["\xffa" => 1, "\xff" . "a" => 2];
//                      ^^^^^^^^^^^^ DuplicateArrayKey: Array key '\xffa' is duplicated — the earlier entry is silently overwritten
    echo count($k);
}
function g(string $s): void {
    if ($s === "\xff" || $s === "\xfe") {
        if ($s === "\xfe") { echo 1; }
    }
}
