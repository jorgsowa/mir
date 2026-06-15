===description===
Bad by ref
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(string &$v): void {}
fooFoo("a");
===expect===
InvalidPassByReference@3:7-3:10: Argument $v of fooFoo() must be passed by reference
