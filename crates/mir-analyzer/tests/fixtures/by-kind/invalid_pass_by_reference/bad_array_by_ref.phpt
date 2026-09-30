===description===
Bad array by ref
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(array &$a): void {}
fooFoo([1, 2, 3]);
//     ^^^^^^^^^ InvalidPassByReference: Argument $a of fooFoo() must be passed by reference
===expect===
