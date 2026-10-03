===description===
No crash with empty match
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function foo(int $i) {
    match ($i) {
//  ^ +2:5 UnhandledMatchCondition: Unhandled match condition: no arms

    };
}
===expect===
