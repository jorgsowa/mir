===description===
`mb_send_mail` hands its fifth argument to the mail command, like `mail`.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UndefinedClass errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
mb_send_mail('a@b.c', 's', 'm', [], $_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
mb_send_mail('a@b.c', 's', $_GET['x']);
