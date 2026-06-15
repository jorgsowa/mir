===description===
reports duplicate named argument
===file===
<?php
function greet(string $name): void {}
greet(name: 'Ada', name: 'Grace');
===expect===
UnusedParam@2:15-2:27: Parameter $name is never used
InvalidNamedArgument@3:19-3:32: greet() has no parameter named $name
