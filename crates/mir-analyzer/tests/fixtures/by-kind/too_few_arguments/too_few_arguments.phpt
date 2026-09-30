===description===
Too few arguments
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(int $a): void {}
fooFoo();
//<^^^^^^^^ TooFewArguments: Too few arguments for fooFoo(): expected 1, got 0
===expect===
