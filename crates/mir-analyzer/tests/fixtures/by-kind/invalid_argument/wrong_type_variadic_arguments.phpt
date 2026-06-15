===description===
Wrong type variadic arguments
===config===
suppress=UnusedParam
===file===
<?php
function takesArguments(int ...$args) : void {}

takesArguments(age: "abc");
===expect===
InvalidArgument@4:15-4:25: Argument $args of takesArguments() expects 'int', got '"abc"'
