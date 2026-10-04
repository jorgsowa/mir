===description===
A first-class callable of an unqualified global function inside a namespace resolves to the global function, unless the namespace declares its own
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Domain;

/**
 * @template K of array-key
 * @template N of array-key
 * @param callable(K): N $transform
 * @param array<K, int> $source
 * @return array<N, int>
 */
function mapKeys(callable $transform, array $source): array
{
    return [];
}

function strtoupper(string $s): int
{
    return 1;
}

/** @param array<string, int> $source */
function build(array $source): void {
    $lowered = mapKeys(mb_strtolower(...), $source);
    /** @mir-check $lowered is array<string, int> */

    $qualified = mapKeys(\mb_strtolower(...), $source);
    /** @mir-check $qualified is array<string, int> */

    $shadowed = mapKeys(strtoupper(...), $source);
    /** @mir-check $shadowed is array<int, int> */

    $missing = mapKeys(not_a_function(...), $source);
}
===expect===
