===description===
Too few arguments
===file===
<?php
function fooFoo(int $a): void {}
fooFoo();
===expect===
TooFewArguments@3:1-3:9: Too few arguments for fooFoo(): expected 1, got 0
