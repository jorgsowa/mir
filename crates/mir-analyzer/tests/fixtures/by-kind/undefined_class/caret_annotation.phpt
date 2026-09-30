===description===
Expectations written as `^^^` annotations under the source line.
===config===
suppress=UnusedParam
===file===
<?php
function f(NotARealClass $x): void {}
//         ^^^^^^^^^^^^^ UndefinedClass: Class NotARealClass does not exist
===expect===
