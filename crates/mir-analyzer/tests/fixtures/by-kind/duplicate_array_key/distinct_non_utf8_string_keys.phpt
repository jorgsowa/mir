===description===
String keys that are not valid UTF-8 stay distinct from each other.
===file===
<?php
function test(string $c): int {
    $map = ["\xC0" => 1, "\xD0" => 2, "\xE0" => 3];
    return $map[$c] ?? 0;
}
