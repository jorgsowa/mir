===description===
array<T> single type param accepts string-keyed arrays (no FP)
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param array<mixed> $arr
 */
function acceptsArray(array $arr): void { var_dump($arr); }

/** @var array<string, int> $data */
$data = ["foo" => 1, "bar" => 2];

acceptsArray($data);
