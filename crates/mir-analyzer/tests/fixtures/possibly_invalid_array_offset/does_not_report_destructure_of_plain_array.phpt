===file===
<?php
/** @return array */
function get(): array { return []; }
function test(): void {
    [$a, $b] = get();
    var_dump($a, $b);
}
===expect===
