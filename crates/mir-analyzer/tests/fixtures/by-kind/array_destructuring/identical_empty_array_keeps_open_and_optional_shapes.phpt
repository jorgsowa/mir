===description===
`$row !== []` must keep shapes that can still be non-empty: a shape whose
keys are all optional, and an open shape. Only the closed empty shape is dropped.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array{a?: int}|array{} $row */
function optional_keys(array $row): void {
    if ($row !== []) {
        $v = $row;
        /** @mir-check $v is array{a?: int} */
        echo 1;
    }
}

/** @param array<string, int>|array{} $row */
function generic_array(array $row): void {
    if ($row !== []) {
        $v = $row;
        /** @mir-check $v is non-empty-array<string, int> */
        echo 1;
    }
}
