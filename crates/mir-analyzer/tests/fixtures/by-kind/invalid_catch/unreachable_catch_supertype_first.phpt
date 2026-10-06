===description===
A catch clause whose type is a subtype of (or identical to) a type already
caught by an EARLIER clause on the same try can never run — the earlier
clause always wins. The try/catch validation only ever checked each catch
type against Throwable, never against previously-listed catch types.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function doSomething(): void {}

function wrong_order(): void {
    try {
        doSomething();
    } catch (\Exception $e) {
    } catch (\InvalidArgumentException $e) {
//           ^^^^^^^^^^^^^^^^^^^^^^^^^ UnreachableCatch: Catch block for 'InvalidArgumentException' is unreachable — already caught by 'Exception'
    }
}

function right_order(): void {
    try {
        doSomething();
    } catch (\InvalidArgumentException $e) {
    } catch (\Exception $e) {
    }
}

function union_catch_shadows_later_subtype(): void {
    try {
        doSomething();
    } catch (\TypeError|\Exception $e) {
    } catch (\InvalidArgumentException $e) {
//           ^^^^^^^^^^^^^^^^^^^^^^^^^ UnreachableCatch: Catch block for 'InvalidArgumentException' is unreachable — already caught by 'Exception'
    }
}
