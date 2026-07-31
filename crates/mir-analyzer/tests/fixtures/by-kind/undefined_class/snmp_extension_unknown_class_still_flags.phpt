===description===
Negative control for the snmp stub vendoring fix (Sector I1): vendoring the
real stub file must not wildcard-resolve unrelated top-level class names —
a class that isn't actually part of the extension must still be flagged
undefined.
===config===
suppress=UnusedParam
===file===
<?php
function f(NotARealSNMPClass $x): void {}
===expect===
UndefinedClass@2:11-2:28: Class NotARealSNMPClass does not exist
