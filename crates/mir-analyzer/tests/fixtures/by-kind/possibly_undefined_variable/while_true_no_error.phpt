===description===
while(true) body assigned before break is not possibly-undefined after loop
===file===
<?php
function foo(callable $cb): mixed {
    $i = 0;
    while (true) {
        $result = $i++;
        if ($result > 3) { break; }
    }
    return $cb($result);
}
===expect===
