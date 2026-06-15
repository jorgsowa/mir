===description===
reports deprecated class instantiation
===file===
<?php
/** @deprecated use NewClass instead */
class OldClass {}

function test(): void {
    $obj = new OldClass();
}
===expect===
UnusedVariable@6:4-6:8: Variable $obj is never read
DeprecatedClass@6:15-6:23: Class OldClass is deprecated: use NewClass instead
