===description===
Calls that connect to the host they receive.
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
ftp_connect($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
ftp_ssl_connect($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
ssh2_connect($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
ldap_connect($_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
socket_connect(socket_create(AF_INET, SOCK_STREAM, 0), $_GET['x']);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSsrf: Tainted URL — possible server-side request forgery
//             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $socket of socket_connect() expects 'Socket', possibly different type 'Socket|false' provided
ftp_connect('a.example');
