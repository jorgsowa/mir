===description===
A case value read from a global or class constant declared in another file is checked against the backing type, as PHP does on first use.
===file:lib.php===
<?php
namespace Lib;

const NAME = 'n';
class K {
    const I = 3;
    const S = 't';
}
===file:enums.php===
<?php
namespace App;

use Lib\K;
use const Lib\NAME;

enum A: int {
    case X = NAME;
//           ^^^^ BackedEnumCaseTypeMismatch: Enum case type string does not match enum backing type int
    case Y = K::S;
//           ^^^^ BackedEnumCaseTypeMismatch: Enum case type string does not match enum backing type int
    case Z = K::I;
}

enum B: string {
    case X = \PHP_INT_MAX;
//           ^^^^^^^^^^^^ BackedEnumCaseTypeMismatch: Enum case type int does not match enum backing type string
    case Y = K::S;
    case Z = NAME . 'x';
}
