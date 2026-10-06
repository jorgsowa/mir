===description===
MissingParamType fires per untyped top-level function parameter; native hints
and docblock @param types both satisfy it.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param string $b
 */
function f($a, $b, int $c): void {}
//         ^^ MissingParamType: Parameter $a of f() has no type annotation
