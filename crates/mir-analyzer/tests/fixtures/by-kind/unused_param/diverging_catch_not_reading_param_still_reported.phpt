===description===
Negative control for the diverging-catch-read-propagation fix: a catch
clause that diverges (rethrows) but never reads the parameter must still
leave it flagged as unused. Guards against an overly broad fix that treats
every diverging catch's whole state as "used", rather than only its actual
reads.
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function h(\ReflectionClass $reflectionClass): void {
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $reflectionClass is never used
    try {
        maybeThrow();
    } catch (\Throwable $e) {
        throw $e;
    }
}

function maybeThrow(): void {}
===expect===
