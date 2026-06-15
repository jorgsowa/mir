===description===
reports invalid named argument
===file===
<?php
function greet(string $name): void {}
greet(who: 'Ada');
===expect===
UnusedParam@2:15-2:27: Parameter $name is never used
InvalidNamedArgument@3:6-3:16: greet() has no parameter named $who
