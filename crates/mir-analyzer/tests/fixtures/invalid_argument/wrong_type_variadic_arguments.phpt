===description===
wrongTypeVariadicArguments
===file===
<?php
function takesArguments(int ...$args) : void {}

takesArguments(age: "abc");
===expect===
InvalidScalarArgument
===ignore===
TODO
