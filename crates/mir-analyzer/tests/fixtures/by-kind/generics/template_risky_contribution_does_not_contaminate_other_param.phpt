===description===
A "fully explained by a sibling alternative" contribution from one param
(the `null` in `T|null` matched against a bare `null` arg) must not
contaminate a DIFFERENT param's real binding for the same template name —
both a false-positive argument error and a masked bound violation.
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
 * @param T|null $a
 * @param T $b
 * @return T
 */
function f($a, $b) {
    return $b;
}

function needsInt(int $x): void {}
needsInt(f(null, 5));

/**
 * @template U of string
 * @param U|null $a
 * @param U $b
 */
function g($a, $b): void {}

g(null, 5);
//<^^^^^^^^^^ InvalidArgument: Argument $a of g() expects 'string', got '5'
