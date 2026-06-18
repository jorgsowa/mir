===description===
while(true) body assigned before break is not possibly-undefined after loop
===config===
suppress=UnusedVariable
===file===
<?php
function foo(callable $cb, int $i): mixed {
    while (true) {
        $result = $i++;
        if ($result > 3) { break; }
    }
    return $cb($result);
}
===expect===
