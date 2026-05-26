===description===
Invalid scalar argument
===file===
<?php
function fooFoo(int $a): void {}
fooFoo("string");
===expect===
InvalidScalarArgument
===ignore===
TODO
