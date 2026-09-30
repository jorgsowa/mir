===description===
unknown param type hint
===config===
suppress=UnusedParam,UnusedFunction
===file===
<?php
function f(UnknownClass $x): void {}
//         ^^^^^^^^^^^^ UndefinedClass: Class UnknownClass does not exist
===expect===
