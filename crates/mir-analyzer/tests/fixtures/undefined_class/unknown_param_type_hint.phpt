===source===
<?php
function f(UnknownClass $x): void {}
===expect===
UndefinedClass at 2:11
# UnusedParam location bug: parameter $x is reported at 1:0 instead of the correct line
UnusedParam at 1:0
