===description===
Throw with message call and assignment and no reference
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function dangerous(): string {
    if (rand(0, 1)) {
        throw new Exception("bad");
    }

    return "hello";
}

function callDangerous(): void {
    $s = null;
//  ^^ UnusedVariable: Variable $s is never read

    try {
        $s = dangerous();
    } catch (Exception $e) {
        echo $e->getMessage();
    }
}
===expect===
