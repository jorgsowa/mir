===description===
Case values read from constants declared in another file are compared with the other cases, as PHP does on first use.
===file:lib.php===
<?php
namespace Lib;

const NAME = 'n';
const ONE = 1;
class K {
    const I = 3;
    const S = 't';
}
===file:enums.php===
<?php
namespace App;

use Lib\K;
use const Lib\NAME;
use const Lib\ONE;

enum A: int {
    case X = ONE;
    case Y = 1;
//           ^ DuplicateEnumCaseValue: Duplicate value in enum App\A for cases X and Y
    case Z = K::I + 1;
    case W = 4;
//           ^ DuplicateEnumCaseValue: Duplicate value in enum App\A for cases Z and W
    case V = 3;
    case U = K::I;
//           ^^^^ DuplicateEnumCaseValue: Duplicate value in enum App\A for cases V and U
}

enum B: string {
    case X = NAME . 'x';
    case Y = 'nx';
//           ^^^^ DuplicateEnumCaseValue: Duplicate value in enum App\B for cases X and Y
    case Z = K::S;
    case W = 'T';
}
