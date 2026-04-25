===file===
<?php
/** @deprecated use NewClass instead */
class OldClass {}

function test(): void {
    $obj = new OldClass();
}
===expect===
UnusedVariable: Variable $obj is never read
DeprecatedClass: Class OldClass is deprecated
