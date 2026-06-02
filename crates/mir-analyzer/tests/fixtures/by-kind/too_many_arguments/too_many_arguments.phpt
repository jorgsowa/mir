===description===
Too many arguments
===file===
<?php
function fooFoo(int $a): void {}
fooFoo(5, "dfd");
===expect===
TooManyArguments@3:11-3:16: Too many arguments for fooFoo(): expected 1, got 2
