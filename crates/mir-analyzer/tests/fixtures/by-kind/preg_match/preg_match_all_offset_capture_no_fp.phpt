===description===
FP-D(a): preg_match_all with PREG_OFFSET_CAPTURE. Accessing $matches[0][0][1] (the
byte offset) must not emit NonExistentArrayOffset.
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

function parseAllOffsets(string $input): array {
    preg_match_all('/\d+/', $input, $matches, PREG_OFFSET_CAPTURE);
    $result = [];
    foreach ($matches[0] as $match) {
        $result[] = $match[1]; // byte offset — valid int access
    }
    return $result;
}
