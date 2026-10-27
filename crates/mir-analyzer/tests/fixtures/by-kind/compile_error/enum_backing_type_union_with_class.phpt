===description===
A class name in a union backing type is qualified with the namespace; `iterable` reads `Traversable|array`.
===file===
<?php
namespace App;

interface I {}

enum E: Foo|iterable|null {
//      ^^^^^^^^^^^^^^^^^ CompileError: Enum backing type must be int or string, App\Foo|Traversable|array|null given [invalid-enum-backing-type]
    case X = 1;
}
