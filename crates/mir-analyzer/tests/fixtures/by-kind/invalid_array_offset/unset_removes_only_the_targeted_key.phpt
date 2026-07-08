===description===
unset($arr['key']) must remove only the targeted key from the tracked
shape, leaving other keys' types untouched.
===file===
<?php
function test(): string {
    $x = ["a" => "value", "b" => "value"];
    unset($x["a"]);
    return $x["b"];
}
===expect===
