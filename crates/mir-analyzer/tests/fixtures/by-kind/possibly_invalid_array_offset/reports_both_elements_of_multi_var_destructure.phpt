===description===
reports both elements of multi var destructure
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
    [$a, $b] = get();
//  ^^^^^^^^^^^^^^^^ PossiblyInvalidArrayOffset: Array offset might be invalid: expects 'array', got 'array|false'
    var_dump($a, $b);
}
