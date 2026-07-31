===description===
Negative control for the mongodb stub vendoring fix (Sector I1): vendoring
the real stub files must not turn `MongoDB\*` into a wildcard-resolved
namespace — a class that isn't actually part of the extension must still
be flagged undefined.
===config===
suppress=UnusedParam
===file===
<?php
function f(MongoDB\Driver\NotARealClass $x): void {}
===expect===
UndefinedClass@2:11-2:39: Class MongoDB\Driver\NotARealClass does not exist
