===description===
Too many arguments
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(int $a): void {}
fooFoo(5, "dfd");
//        ^^^^^ TooManyArguments: Too many arguments for fooFoo(): expected 1, got 2
===expect===
