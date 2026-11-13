===description===
A sentence-ending dot after a `@return` type isn't part of the type.
===file===
<?php
namespace App;

final class F {
    /** @return void. */
    public function run() {}
}

/** @return int. */
function g() { return "x"; }
//             ^^^^^^^^^^^ InvalidReturnType: Return type '"x"' is not compatible with declared 'int'
