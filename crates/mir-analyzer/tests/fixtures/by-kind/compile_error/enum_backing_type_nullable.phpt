===description===
A nullable backing type is PHP's compile error, not a parse error.
===file===
<?php
namespace App;

interface I {}

enum E: ?int {
//      ^^^^ CompileError: Enum backing type must be int or string, ?int given [invalid-enum-backing-type]
    case X = 1;
}
