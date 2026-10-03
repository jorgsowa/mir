===description===
Only zero-arg calls are memoized: a call with arguments is not assumed to
return the same value twice.
===config===
<mir memoizeMethodCallResults="true">
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Box {
    public function find(int $id): ?string { return null; }
    public function name(): ?string { return null; }
}
function f(Box $b): string {
    if ($b->find(1) === null) {
        return '';
    }
    return $b->find(1);
//  ^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'string|null' is not compatible with declared 'string'
}
function g(Box $b): string {
    if ($b->name() === null) {
        return '';
    }
    return $b->name();
}
===expect===
