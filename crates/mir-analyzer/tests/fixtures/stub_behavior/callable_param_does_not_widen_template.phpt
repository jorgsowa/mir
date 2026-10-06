===description===
A callable's parameter type only binds a template no other argument bound, so a nullable closure parameter does not widen a key bound to a string
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template TK of array-key
 * @template TN of array-key
 * @param callable(TK): TN $transform
 * @param array<TK, int> $source
 * @return array<TN, int>
 */
function mapKeys(callable $transform, array $source): array
{
    return [];
}

/**
 * @template T of int
 * @param callable(T): void $visit
 * @return T
 */
function visit(callable $visit): int
{
    return 0;
}

/** @param array<string, int> $source */
function run(array $source): void {
    $nullable = mapKeys(fn(?string $s, ?string $encoding = null): string => (string) $s, $source);
    /** @mir-check $nullable is array<string, int> */

    $callableFirst = mapKeys(strtolower(...), $source);
    /** @mir-check $callableFirst is array<string, int> */

    $only = visit(fn(int $i): int => $i);
    /** @mir-check $only is int */

    visit(fn(string $s): string => $s);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'string' does not satisfy bound 'int'
}
===expect===
