===description===
sscanf multiple output vars are defined
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function parse_pair(string $input): int {
    sscanf($input, '%d %d', $left, $right);
    return $left + $right;
}
