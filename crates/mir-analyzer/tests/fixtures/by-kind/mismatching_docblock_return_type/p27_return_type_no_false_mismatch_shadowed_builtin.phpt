===description===
P27: a same-namespace class shadowing a builtin iterator-family name (P25's family)
must not produce a false `MismatchingDocblockReturnType` when both the docblock
`@return` and the native return-type hint bare-name the shadowed class. The native
hint always resolves strictly (`App\Generator`); the docblock side is intentionally
left lenient at collection time (`Generator`, matching Psalm/PHPStan's bare-builtin
leniency) since it can't yet know whether a local class shadows the name. Comparing
the two without reconciling would treat a real, intentional match as a contradiction.
===file===
<?php

namespace App;

final class Generator
{
    public function build(): string { return 'built'; }
}

/**
 * @return Generator
 */
function make(): Generator
{
    return new Generator();
}
===expect===
