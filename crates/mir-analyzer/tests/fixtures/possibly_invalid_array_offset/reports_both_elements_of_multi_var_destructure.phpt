===description===
reports both elements of multi var destructure
===file===
<?php
/** @return array|false */
function get(): array|false { return false; }
function test(): void {
    [$a, $b] = get();
    var_dump($a, $b);
}
===expect===
PossiblyInvalidArrayOffset: Array offset might be invalid: expects 'array', got 'array<mixed, mixed>|false'
===ignore===
TODO
