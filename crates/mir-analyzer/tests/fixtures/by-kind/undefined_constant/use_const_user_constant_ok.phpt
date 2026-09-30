===description===
`use const` resolves a user-defined namespaced constant, with and without an alias.
===file===
<?php
namespace Lib {
    const LIMIT = 10;
}

namespace App {
    use const Lib\LIMIT;
    use const Lib\LIMIT as MAX;

    function limits(): int {
        $a = LIMIT;
        $b = MAX;
        /** @mir-check $a is int */
        /** @mir-check $b is int */
        return $a + $b;
    }
}
===expect===
