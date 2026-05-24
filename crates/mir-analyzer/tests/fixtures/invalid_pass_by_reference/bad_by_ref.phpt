===description===
badByRef
===file===
<?php
function fooFoo(string &$v): void {}
fooFoo("a");
===expect===
InvalidPassByReference
===ignore===
TODO
