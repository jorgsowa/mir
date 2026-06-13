===description===
Bad by ref
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(string &$v): void {}
fooFoo("a");
===expect===
InvalidPassByReference@3:8-3:11: Argument $v of fooFoo() must be passed by reference
