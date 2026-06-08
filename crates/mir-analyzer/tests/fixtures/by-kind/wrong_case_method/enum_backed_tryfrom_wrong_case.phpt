===description===
Wrong-case call to synthesized backed-enum tryFrom() is reported with canonical camelCase name, not the lowercased key.
===file===
<?php
enum Color: string {
    case Red = 'red';
    case Blue = 'blue';
}
Color::TRYFROM('red');
===expect===
WrongCaseMethod@6:8-6:15: Method name 'Color::TRYFROM' has incorrect casing; use 'tryFrom'
