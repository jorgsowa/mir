===description===
All matched another impossible
===file===
<?php
function foo() : string {
    $a = rand(0, 1) ? "a" : "b";
    return match ($a) {
        "a" => "hello",
        "b" => "goodbye",
        "c" => "impossible",
//      ^^^ TypeDoesNotContainType: Type '"a"|"b"' can never contain type '"c"'
    };
}
