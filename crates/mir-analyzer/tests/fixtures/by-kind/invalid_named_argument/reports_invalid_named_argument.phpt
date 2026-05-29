===description===
reports invalid named argument
===file===
<?php
function greet(string $name): void {}
greet(who: 'Ada');
===expect===
UnusedParam@2:16-2:28: Parameter $name is never used
InvalidNamedArgument@3:7-3:17: greet() has no parameter named $who
