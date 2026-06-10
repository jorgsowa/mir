===description===
All matched another impossible
===ignore===
TODO
===file===
<?php
function foo() : string {
    $a = rand(0, 1) ? "a" : "b";
    return match ($a) {
        "a" => "hello",
        "b" => "goodbye",
        "c" => "impossible",
    };
}
===expect===
