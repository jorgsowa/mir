===description===
`use const` of a namespaced extension constant resolves an unqualified
reference in another namespace.
===file===
<?php
namespace App;

use const ast\AST_CLASS;

function kind(): int {
    $k = AST_CLASS;
    /** @mir-check $k is int */
    return $k;
}
