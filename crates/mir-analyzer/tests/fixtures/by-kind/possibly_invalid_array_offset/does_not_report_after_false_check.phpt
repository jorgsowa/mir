===description===
does not report after false check
===config===
suppress=ForbiddenCode,MixedAssignment
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
