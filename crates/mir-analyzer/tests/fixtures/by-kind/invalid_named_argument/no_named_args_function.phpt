===description===
No named args function
===file===
<?php
/** @no-named-arguments */
function takesArguments(string $name, int $age) : void {}

takesArguments(age: 5, name: "hello");
===expect===
NamedArgumentNotAllowed
===ignore===
TODO
