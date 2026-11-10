===description===
`#[\NoDiscard]` has no effect before PHP 8.5.
===config===
<mir>
  <issueHandlers>
    <UndefinedAttributeClass errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
#[\NoDiscard]
function mint(): int
{
    return 1;
}

mint();
