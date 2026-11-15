===description===
A password checked to be non-empty, and a password from the program, aren't
reported.
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
function login(LDAP\Connection $conn, string $dn, string $servicePassword): bool {
    $password = (string) $_POST['password'];
    if ($password === '') {
        return false;
    }
    if (!ldap_bind($conn, $dn, $password)) {
        return false;
    }
    $other = (string) $_POST['other'];
    if (empty($other)) {
        return false;
    }
    ldap_bind($conn, $dn, $other);
    $third = (string) $_POST['third'];
    if (strlen($third) < 8) {
        return false;
    }
    ldap_bind($conn, $dn, $third);
    return ldap_bind($conn, 'cn=service', $servicePassword);
}
