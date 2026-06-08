===description===
Wrong-case call to synthesized backed-enum tryFrom() is reported with canonical camelCase name, not the lowercased key.
===file===
<?php
enum Color: string {
    case Red = 'red';
    case Blue = 'blue';
}
$x = Color::TRYFROM('red');
===expect===
WrongCaseMethod@6:13-6:20: Method name 'Color::TRYFROM' has incorrect casing; use 'tryFrom'
