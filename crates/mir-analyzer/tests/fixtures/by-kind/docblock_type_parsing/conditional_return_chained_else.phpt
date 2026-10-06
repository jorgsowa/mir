===description===
A conditional return whose else-branch is another conditional resolves each level, inside a parenthesised group with a `|null` suffix.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace Lib;

/**
 * @template T
 * @param T $x
 * @return (T is int ? int : T is string ? string : float)|null
 */
function pick($x) {
    return null;
}

/**
 * @return ($flag is true ? int : $flag is false ? string : float)
 */
function by_flag(bool $flag): int|string|float {
    return 1;
}

function check(): void {
    /** @mir-check pick(1) is int|null */
    pick(1);
    /** @mir-check pick('s') is string|null */
    pick('s');
    /** @mir-check pick(1.5) is float|null */
    pick(1.5);
    /** @mir-check by_flag(true) is int */
    by_flag(true);
    /** @mir-check by_flag(false) is string */
    by_flag(false);
}
