===file===
<?php
function greet(string $name): void {}
greet(name: 'Ada', name: 'Grace');
===expect===
UnusedParam: Parameter $name is never used
InvalidNamedArgument: greet() has no parameter named $name
