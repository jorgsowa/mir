===description===
Named groups give a shape keyed by name and number, compatible with a declared return shape.
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

/**
 * @throws Exception
 * @return array{year: string, month: string} */
function parse(string $s): array {
    if (!preg_match('/(?<year>\d+)-(?<month>\d+)/', $s, $m)) {
        throw new Exception();
    }
    return $m;
}

===expect===
