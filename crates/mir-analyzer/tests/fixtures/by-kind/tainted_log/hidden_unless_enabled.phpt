===description===
`TaintedLog` is opt-in: without an `<issueHandlers>` entry giving it a level,
nothing is reported.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function login(): void {
    error_log('Login attempt: ' . $_POST['user']);
}
