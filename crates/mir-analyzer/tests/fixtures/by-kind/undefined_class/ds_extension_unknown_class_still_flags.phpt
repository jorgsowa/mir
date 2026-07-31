===description===
Negative control for the ds stub vendoring fix (Sector I1): vendoring the
real stub file must not turn `Ds\*` into a wildcard-resolved namespace — a
class that isn't actually part of the extension must still be flagged
undefined.
===config===
suppress=UnusedParam
===file===
<?php
function f(Ds\NotARealClass $x): void {}
===expect===
UndefinedClass@2:11-2:27: Class Ds\NotARealClass does not exist
