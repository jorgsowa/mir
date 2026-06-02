===description===
Bad array by ref
===file===
<?php
function fooFoo(array &$a): void {}
fooFoo([1, 2, 3]);
===expect===
InvalidPassByReference@3:8-3:17: Argument $a of fooFoo() must be passed by reference
