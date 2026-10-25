===description===
An interface listed twice is a compile error, case-insensitively; an enum already implements UnitEnum, and BackedEnum when backed. Re-implementing a parent's interface is allowed.
===file===
<?php
namespace App;

interface Shape {}

class Square implements Shape, \App\shape {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ WrongCaseClass: Class name 'shape' has incorrect casing; use 'Shape'
//                             ^^^^^^^^^^ CompileError: Class App\Square cannot implement previously implemented interface App\Shape [duplicate-interface]

interface Polygon extends Shape, Shape {}
//                               ^^^^^ CompileError: Interface App\Polygon cannot implement previously implemented interface App\Shape [duplicate-interface]

enum Suit implements \UnitEnum {
//                   ^^^^^^^^^ CompileError: Enum App\Suit cannot implement previously implemented interface UnitEnum [duplicate-interface]
    case Hearts;
}

enum Size: int implements \BackedEnum {
//                        ^^^^^^^^^^^ CompileError: Enum App\Size cannot implement previously implemented interface BackedEnum [duplicate-interface]
    case Small = 1;
}

class Base implements Shape {}

class Child extends Base implements Shape {}
