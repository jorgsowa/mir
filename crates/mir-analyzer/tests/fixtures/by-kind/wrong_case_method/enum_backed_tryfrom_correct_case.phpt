===description===
Correct-case calls to synthesized backed-enum methods (tryFrom/from/cases) produce no WrongCaseMethod.
===file===
<?php
enum Color: string {
    case Red = 'red';
    case Blue = 'blue';
}
Color::tryFrom('red');
Color::from('blue');
Color::cases();
===expect===
