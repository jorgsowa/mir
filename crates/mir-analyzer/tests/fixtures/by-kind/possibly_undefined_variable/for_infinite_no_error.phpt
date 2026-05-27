===description===
for(;;) body assigned before break is not possibly-undefined after loop
===file===
<?php
function foo(callable $cb): mixed {
    $i = 0;
    for (;;) {
        $result = $i++;
        if ($result > 3) { break; }
    }
    return $cb($result);
}
===expect===
