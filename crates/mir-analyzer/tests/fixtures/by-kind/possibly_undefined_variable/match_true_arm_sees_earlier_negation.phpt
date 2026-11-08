===description===
A later `match (true)` arm runs only when the earlier arms' boolean conditions were false.
===file===
<?php
function f(): string|false {
    if (rand()) {
        $v = (string) rand();
    }
    return match (true) {
        !isset($v) => false,
        (int) $v >= 6000 => $v,
        default => 'x',
    };
}
