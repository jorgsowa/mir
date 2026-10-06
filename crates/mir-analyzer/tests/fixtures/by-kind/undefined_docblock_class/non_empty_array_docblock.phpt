===description===
`non-empty-array` resolves as a docblock keyword.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-empty-array $xs */
function useList(array $xs): void {
    /** @mir-check $xs is non-empty-array */
    $_ = 1;
}

/** @return non-empty-array */
function makeList(): array {
    return [1, 2];
}

/** @return non-empty-array */
function makeEmptyList(): array {
    return [];
//  ^^^^^^^^^^ InvalidReturnType: Return type 'array{}' is not compatible with declared 'non-empty-array'
}

useList([1]);
useList([]);
//      ^^ ArgumentTypeCoercion: Argument $xs of useList() expects 'non-empty-array', got 'array{}' — coercion may fail at runtime
