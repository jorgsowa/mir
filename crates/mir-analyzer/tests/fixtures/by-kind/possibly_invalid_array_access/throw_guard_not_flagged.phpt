===description===
PossiblyInvalidArrayAccess does NOT fire after a throw-based guard narrows an
int|array parameter to pure array — the int branch is excluded by the early
throw on the non-array path.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function process(int|array $data): void {
    if (!is_array($data)) {
        throw new \InvalidArgumentException('expected array');
    }
    echo $data[0];
}
===expect===
