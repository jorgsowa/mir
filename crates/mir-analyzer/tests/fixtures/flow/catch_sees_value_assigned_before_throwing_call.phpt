===description===
catch sees a value assigned partway through try, before a call that may throw
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function make(): object { return new stdClass(); }

function f(): void {
    $c = false;
    try {
        $c = null;
        $c = make();
    } catch (Exception) {
        echo null === $c ? 'none' : 'some';
    }
}
