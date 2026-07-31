===description===
Negative control for the relay stub vendoring fix (Sector I1): vendoring
the real stub files must not turn `Relay\*` into a wildcard-resolved
namespace — a class that isn't actually part of the extension must still
be flagged undefined.
===config===
suppress=UnusedParam
===file===
<?php
function f(Relay\NotARealClass $x): void {}
===expect===
UndefinedClass@2:11-2:30: Class Relay\NotARealClass does not exist
