===description===
does not report destructure of plain array
===config===
suppress=ForbiddenCode,MixedAssignment
===file===
<?php
/** @return array */
function get(): array { return []; }
function test(): void {
    [$a, $b] = get();
    var_dump($a, $b);
}
===expect===
