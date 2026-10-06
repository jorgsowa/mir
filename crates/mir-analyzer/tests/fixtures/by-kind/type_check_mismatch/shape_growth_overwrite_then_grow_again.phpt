===description===
Growing a shape with a new key and overwriting an already-known key compose
correctly: overwriting 'a' twice must not add a duplicate property, and a
later new key ('b') still grows the shape rather than generalizing it.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, int> $counts */
function test(array $counts): void {
    if ($counts === []) {
        $counts['a'] = 1;
        $counts['a'] = 2;
        $counts['b'] = 3;
        /** @mir-check $counts is array{'a': 2, 'b': 3} */
        $_ = $counts;
    }
}
