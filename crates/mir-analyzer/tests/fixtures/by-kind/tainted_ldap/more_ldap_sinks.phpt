===description===
A distinguished name built from client data selects another entry. Bound values aren't part of the DN.
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
function dn($c): void {
    ldap_bind($c, $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLdap: Tainted LDAP query — possible LDAP injection
    ldap_add($c, $_GET['x'], []);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLdap: Tainted LDAP query — possible LDAP injection
    ldap_modify($c, $_GET['x'], []);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLdap: Tainted LDAP query — possible LDAP injection
    ldap_mod_add($c, $_GET['x'], []);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLdap: Tainted LDAP query — possible LDAP injection
    ldap_mod_del($c, $_GET['x'], []);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLdap: Tainted LDAP query — possible LDAP injection
    ldap_mod_replace($c, $_GET['x'], []);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLdap: Tainted LDAP query — possible LDAP injection
    ldap_rename($c, $_GET['x'], 'y', 'z', true);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedLdap: Tainted LDAP query — possible LDAP injection
}

function values($c): void {
    ldap_bind($c, 'cn=a', $_GET['password']);
    ldap_add($c, 'cn=a', ['cn' => $_GET['x']]);
}
