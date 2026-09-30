===description===
reports duplicate named argument
===file===
<?php
function greet(string $name): void {}
//             ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used
greet(name: 'Ada', name: 'Grace');
//                 ^^^^^^^^^^^^^ InvalidNamedArgument: greet() has no parameter named $name
===expect===
