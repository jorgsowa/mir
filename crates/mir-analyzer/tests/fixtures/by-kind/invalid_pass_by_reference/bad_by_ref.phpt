===description===
Bad by ref
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(string &$v): void {}
fooFoo("a");
//     ^^^ InvalidPassByReference: Argument $v of fooFoo() must be passed by reference
===expect===
