===file===
<?php
/** @return array|false */
function get(): array|false { return false; }
function test(): void {
    $r = get();
    if ($r !== false) {
        [$a] = $r;
        var_dump($a);
    }
}
===expect===
