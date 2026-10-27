===description===
A union backing type is PHP's compile error, its members in PHP's order.
===file===
<?php
namespace App;

interface I {}

enum E: int|string implements I {
//      ^^^^^^^^^^ CompileError: Enum backing type must be int or string, string|int given [invalid-enum-backing-type]
    case X = 1;
}
