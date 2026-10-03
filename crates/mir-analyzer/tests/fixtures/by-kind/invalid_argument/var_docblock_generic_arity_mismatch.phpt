===description===
`@var TypedMap<string>` against a class declaring 2 `@template` params
(`<K, V>`) is now flagged as an arity mismatch — a bare `@var TypedMap`
with no type args at all stays silent (the legitimate bare-generic-reference
shorthand), and a fully and correctly supplied arg list stays silent too.
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
 * @template K
 * @template V
 */
class TypedMap {}

function test_too_few_type_args(): void {
    /** @var TypedMap<string> $m */
    $m = new TypedMap();
//  ^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: TypedMap expects 2 template argument(s), got 1
}

function test_too_many_type_args(): void {
    /** @var TypedMap<string, int, bool> $m */
    $m = new TypedMap();
//  ^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: TypedMap expects 2 template argument(s), got 3
}

function test_bare_generic_reference_stays_silent(): void {
    /** @var TypedMap $m */
    $m = new TypedMap();
//  ^^^^^^^^^^^^^^^^^^^^ UnnecessaryVarAnnotation: @var annotation for $m is unnecessary
}

function test_correct_arity_stays_silent(): void {
    /** @var TypedMap<string, int> $m */
    $m = new TypedMap();
}
===expect===
