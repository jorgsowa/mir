===description===
reports destructure of array or false
===config===
suppress=ForbiddenCode,MixedAssignment
===file===
<?php
/** @return array|false */
function get(): array|false { return false; }
function test(): void {
    [$a, $b] = get();
    var_dump($a, $b);
}
===expect===
PossiblyInvalidArrayOffset@5:4-5:20: Array offset might be invalid: expects 'array', got 'array|false'
