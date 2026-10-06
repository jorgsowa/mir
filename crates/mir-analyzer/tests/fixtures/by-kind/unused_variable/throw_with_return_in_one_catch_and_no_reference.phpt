===description===
Throw with return in one catch and no reference
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class E1 extends Exception {}

function dangerous(): void {
    if (rand(0, 1)) {
        throw new Exception("bad");
    }
}

function callDangerous(): void {
    try {
        dangerous();
        $s = true;
//      ^^ UnusedVariable: Variable $s is never read
    } catch (E1 $e) {
        echo $e->getMessage();
        $s = false;
    } catch (Exception $e) {
        return;
    }
}
