===description===
Negative control for the amqp stub vendoring fix (Sector I1): vendoring the
real stub file must not wildcard-resolve unrelated top-level class names —
a class that isn't actually part of the extension must still be flagged
undefined.
===config===
suppress=UnusedParam
===file===
<?php
function f(NotARealAMQPClass $x): void {}
===expect===
UndefinedClass@2:11-2:28: Class NotARealAMQPClass does not exist
