===description===
Too many arguments
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(int $a): void {}
fooFoo(5, "dfd");
===expect===
TooManyArguments@3:10-3:15: Too many arguments for fooFoo(): expected 1, got 2
