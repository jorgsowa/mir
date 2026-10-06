===description===
FP: preg_match_all with PREG_OFFSET_CAPTURE — accessing $matches[0][0][0] (string)
and $matches[0][0][1] (int) must not emit NonExistentArrayOffset.
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

function extractOffsets(string $s): array {
    preg_match_all('/(\d+)/', $s, $matches, PREG_OFFSET_CAPTURE);
    $offsets = [];
    foreach ($matches[0] as $entry) {
        $offsets[] = $entry[1]; // int byte offset
    }
    return $offsets;
}
