===description===
A general `int` passed where a template is bound by `positive-int` is a coercion.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T of positive-int
 * @param T $n
 * @return T
 */
function page(int $n): int { return $n; }

function run(int $n): void {
    page(3);
    page($n);
//  ^^^^^^^^ ArgumentTypeCoercion: Argument $n of page() expects 'positive-int', got 'int' — coercion may fail at runtime
}
