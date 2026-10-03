===description===
Closure(T): R parameter binds T and R from a typed closure argument
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T
 * @template R
 * @param Closure(T): R $fn
 * @param T $value
 * @return R
 */
function apply(callable $fn, mixed $value): mixed { return $fn($value); }

$result = apply(fn(string $s): int => strlen($s), 'hello');
/** @mir-check $result is int */
===expect===
