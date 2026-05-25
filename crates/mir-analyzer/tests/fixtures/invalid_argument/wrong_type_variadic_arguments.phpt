===description===
Wrong type variadic arguments
===file===
<?php
function takesArguments(int ...$args) : void {}

takesArguments(age: "abc");
===expect===
InvalidScalarArgument
===ignore===
TODO
