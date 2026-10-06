===description===
FP-D(a): preg_match with PREG_OFFSET_CAPTURE. Accessing $matches[0][1] (the byte
offset) must not emit NonExistentArrayOffset. Before the fix, $matches was typed as
list<string>, so subscripting $matches[0] returned string and $matches[0][1] emitted
NonExistentArrayOffset (string has no int-keyed index 1).
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedFunction errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

function parseOffset(string $input): int {
    if (preg_match('/\d+/', $input, $matches, PREG_OFFSET_CAPTURE)) {
        return $matches[0][1]; // byte offset — valid int access
    }
    return -1;
}
