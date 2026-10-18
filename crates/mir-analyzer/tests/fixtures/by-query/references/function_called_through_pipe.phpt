===description===
A function used as a first-class callable on the right of `|>` is referenced there.
===cursor===
references include_declaration
===file===
<?php
function dou<CURSOR>ble(int $x): int { return $x * 2; }
echo 5 |> double(...);
echo 5 |> double(...) |> double(...);
===expect===
test.php@2:9-2:15
test.php@3:10-3:16
test.php@4:10-4:16
test.php@4:25-4:31
