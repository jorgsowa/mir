===description===
A @param naming a missing class does not cascade into InvalidArgument when the parameter is forwarded.
===file===
<?php
namespace App;

/** @param Missing $x */
function forward($x): int {
//       ^^^^^^^ UndefinedDocblockClass: Docblock type 'App\Missing' does not exist
    return strlen($x);
}
===expect===
