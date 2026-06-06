===description===
Wrong case interface name in enum implements is reported.
===file===
<?php
interface Stringable2 {}
enum Color: string implements stringable2 {
    case Red = 'red';
}
===expect===
WrongCaseClass@3:0-3:43: Class name 'stringable2' has incorrect casing; use 'Stringable2'
