===description===
InvalidCatch does NOT fire for \Error or its subclasses, which implement Throwable via the Error hierarchy.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
try {
    throw new \Error("fail");
} catch (\Error $e) {
    echo $e->getMessage();
} catch (\TypeError $e) {
//       ^^^^^^^^^^ UnreachableCatch: Catch block for 'TypeError' is unreachable — already caught by 'Error'
    echo $e->getMessage();
}
