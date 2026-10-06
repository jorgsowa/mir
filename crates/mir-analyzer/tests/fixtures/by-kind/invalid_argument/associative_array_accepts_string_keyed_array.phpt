===description===
`associative-array` accepts matching string-keyed arrays.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param associative-array<string, int> $items
 */
function takesAssociativeArray($items): void {}

takesAssociativeArray(['x' => 1, 'y' => 2]);
