===description===
does not report destructure of plain array
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @return array */
function get(): array { return []; }
function test(): void {
    [$a, $b] = get();
    var_dump($a, $b);
}
===expect===
