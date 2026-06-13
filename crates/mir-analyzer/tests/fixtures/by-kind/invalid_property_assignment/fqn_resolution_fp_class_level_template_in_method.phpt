===description===
FP guard: class-level template params referenced in method return types must not be namespace-qualified
===config===
suppress=UnusedVariable
===file:Lib/Collection.php===
<?php
namespace Lib;

/**
 * @template TKey
 * @template TVal
 */
class Collection {
    /**
     * @return TKey|null
     */
    public function firstKey(): mixed {
        return null;
    }

    /**
     * @return TVal|null
     */
    public function first(): mixed {
        return null;
    }
}
===file:App/UseIt.php===
<?php
namespace App;

use Lib\Collection;

/** @var Collection<string, int> $col */
$col = new Collection();
$key = $col->firstKey();
/** @mir-check $key is string|null */
$val = $col->first();
/** @mir-check $val is int|null */
===expect===
