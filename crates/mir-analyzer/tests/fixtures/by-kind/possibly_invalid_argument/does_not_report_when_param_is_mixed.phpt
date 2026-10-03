===description===
does not report when param is mixed
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function takesMixed(mixed $v): void { var_dump($v); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    takesMixed(getResult());
}
===expect===
