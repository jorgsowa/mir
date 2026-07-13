===description===
while(1) is an infinite loop, same as while(true) — a variable assigned
before every break is not possibly-undefined after the loop.
===config===
suppress=UnusedVariable
===file===
<?php
function foo(callable $cb, int $i): mixed {
    while (1) {
        $result = $i++;
        if ($result > 3) { break; }
    }
    return $cb($result);
}
===expect===
