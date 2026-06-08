===description===
Correct-case calls to synthesized backed-enum methods (tryFrom/from/cases) produce no WrongCaseMethod.
===file===
<?php
enum Color: string {
    case Red = 'red';
    case Blue = 'blue';
}
$x = Color::tryFrom('red');
$y = Color::from('blue');
$z = Color::cases();
===expect===
