===description===
An enum-case subject (`T is Key::A`) cannot be decided from the argument type, so the conditional widens to both branches without a bogus class or ImplicitToStringCast.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
enum Key { case A; case B; }

/**
 * @template T of Key
 * @param T $k
 * @return (T is Key::A ? int : string)|null
 */
function get(Key $k) { return null; }

function test(): void {
    $v = get(Key::B);
    /** @mir-check $v is int|string|null */
    echo "x$v";
}
===expect===
