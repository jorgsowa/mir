===description===
allMatchedDefaultImpossible
===file===
<?php
function foo() : string {
    $a = rand(0, 1) ? "a" : "b";
    return match ($a) {
        "a" => "hello",
        "b" => "goodbye",
        default => "impossible",
    };
}
===expect===
TypeDoesNotContainType
===ignore===
TODO
