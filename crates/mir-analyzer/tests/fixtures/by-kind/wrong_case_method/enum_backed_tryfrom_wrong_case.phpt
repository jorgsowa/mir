===description===
Wrong-case call to synthesized backed-enum tryFrom() is reported with canonical camelCase name, not the lowercased key.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
enum Color: string {
    case Red = 'red';
    case Blue = 'blue';
}
$x = Color::TRYFROM('red');
===expect===
WrongCaseMethod@6:12-6:19: Method name 'Color::TRYFROM' has incorrect casing; use 'tryFrom'
