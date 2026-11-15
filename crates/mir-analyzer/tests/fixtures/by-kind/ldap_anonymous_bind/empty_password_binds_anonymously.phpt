===description===
A request password passed to `ldap_bind()` without a non-empty check lets an empty
password bind anonymously, which the server reports as a success.
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
function login(LDAP\Connection $conn, string $dn): bool {
    return ldap_bind($conn, $dn, $_POST['password']);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ LdapAnonymousBind: ldap_bind() with a request password that may be empty — an empty password is an anonymous bind, which succeeds
}

function loginTrimmed(LDAP\Connection $conn, string $dn): bool {
    $password = trim((string) $_POST['password']);
    return ldap_bind($conn, $dn, $password);
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ LdapAnonymousBind: ldap_bind() with a request password that may be empty — an empty password is an anonymous bind, which succeeds
}
