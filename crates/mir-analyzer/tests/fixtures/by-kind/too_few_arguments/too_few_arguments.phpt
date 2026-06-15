===description===
Too few arguments
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(int $a): void {}
fooFoo();
===expect===
TooFewArguments@3:0-3:8: Too few arguments for fooFoo(): expected 1, got 0
