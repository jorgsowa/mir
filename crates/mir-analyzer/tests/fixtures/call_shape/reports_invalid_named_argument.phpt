===file===
<?php
function greet(string $name): void {}
greet(who: 'Ada');
===expect===
UnusedParam: Parameter $name is never used
InvalidNamedArgument: greet() has no parameter named $who
