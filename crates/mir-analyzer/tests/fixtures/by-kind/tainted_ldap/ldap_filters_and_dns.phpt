===description===
The base DN and filter of an LDAP search, and the DN of `ldap_delete()`, are
LDAP injection sinks. `ldap_escape()` sanitizes them.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function search(LDAP\Connection $ldap): void {
    ldap_search($ldap, 'dc=example,dc=com', '(uid=' . $_GET['user'] . ')');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLdap: Tainted LDAP query — possible LDAP injection
    ldap_list($ldap, 'ou=' . $_GET['unit'] . ',dc=example,dc=com', '(objectClass=*)');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLdap: Tainted LDAP query — possible LDAP injection
    ldap_delete($ldap, 'uid=' . $_GET['user'] . ',dc=example,dc=com');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLdap: Tainted LDAP query — possible LDAP injection
    ldap_search($ldap, 'dc=example,dc=com', '(uid=' . ldap_escape($_GET['user'], '', LDAP_ESCAPE_FILTER) . ')');
    ldap_search($ldap, 'dc=example,dc=com', '(uid=admin)', [$_GET['attribute']]);
}
