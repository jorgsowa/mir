===description===
reports invalid named argument
===file===
<?php
function greet(string $name): void {}
//             ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used
greet(who: 'Ada');
//    ^^^^^^^^^^ InvalidNamedArgument: greet() has no parameter named $who
===expect===
