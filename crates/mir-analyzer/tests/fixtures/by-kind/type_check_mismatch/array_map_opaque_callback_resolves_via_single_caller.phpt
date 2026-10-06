===description===
array_map's element type resolves through an opaque `callable $cb` parameter
by looking at the concrete closure a caller actually passes.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function apply(callable $cb, array $nums): array {
    $result = array_map($cb, $nums);
    /** @mir-check $result is array<array-key, string> */
    return $result;
}

function useIt(array $nums): void {
    apply(fn(int $x): string => (string) $x, $nums);
}
