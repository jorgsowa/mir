===description===
FN: a template's default referencing another template (`@template TReturn =
T`) inserted the raw, unresolved template atom verbatim instead of resolving
it against the bindings already computed for T — the concrete argument's
type was lost and TReturn stayed an unresolved template, degrading to
`mixed` downstream instead of the argument's real inferred type.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T
 * @template TReturn = T
 * @param T $x
 * @return TReturn
 */
function identity($x) {
    return $x;
}

function needsString(string $s): void {}

needsString(identity(5));
//          ^^^^^^^^^^^ ArgumentTypeCoercion: Argument $s of needsString() expects 'string', got '5' — coercion may fail at runtime
===expect===
