===description===
does not report after false check
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @return array|false */
function get(): array|false { return false; }
function test(): void {
    $r = get();
    if ($r !== false) {
        [$a] = $r;
        var_dump($a);
    }
}
===expect===
