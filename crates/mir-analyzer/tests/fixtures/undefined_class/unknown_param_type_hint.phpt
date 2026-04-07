===source===
<?php
function f(UnknownClass $x): void {}
===expect===
UnusedParam: $x
UndefinedClass: UnknownClass
