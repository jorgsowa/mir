===description===
Wrong type variadic arguments
===config===
suppress=UnusedParam
===file===
<?php
function takesArguments(int ...$args) : void {}

takesArguments(age: "abc");
//             ^^^^^^^^^^ InvalidArgument: Argument $args of takesArguments() expects 'int', got '"abc"'
===expect===
