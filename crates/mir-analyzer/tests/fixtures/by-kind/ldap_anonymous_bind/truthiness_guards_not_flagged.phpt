===description===
Rejecting an empty password with `empty()` or a falsy check rules out the
anonymous bind, even for an untyped request value.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function loginIfNotEmpty(\LDAP\Connection $ldap): bool {
    $password = $_POST['password'];
    if (empty($password)) {
        return false;
    }
    return ldap_bind($ldap, 'uid=alice', $password);
}

function loginIfTruthy(\LDAP\Connection $ldap): bool {
    $password = $_POST['password'];
    if (!$password) {
        return false;
    }
    return ldap_bind($ldap, 'uid=alice', $password);
}
