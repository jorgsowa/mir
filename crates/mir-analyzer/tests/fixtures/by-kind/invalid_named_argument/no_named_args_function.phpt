===description===
No named args function
===config===
suppress=UnusedParam
===file===
<?php
/** @no-named-arguments */
function takesArguments(string $name, int $age) : void {}

takesArguments(age: 5, name: "hello");
===expect===
InvalidNamedArguments@5:15-5:21: takesArguments() does not accept named arguments
InvalidNamedArguments@5:23-5:36: takesArguments() does not accept named arguments
