===description===
No named args function
===file===
<?php
/** @no-named-arguments */
function takesArguments(string $name, int $age) : void {}

takesArguments(age: 5, name: "hello");
===expect===
InvalidNamedArguments@5:16-5:22: takesArguments() does not accept named arguments
InvalidNamedArguments@5:24-5:37: takesArguments() does not accept named arguments
