===description===
@taint-sink on a variadic parameter (`...$args`) only ever checked the
FIRST variadic call-site argument's position — a later variadic argument
silently bypassed the check entirely, in both the plain-function and
method-call taint-sink paths.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @taint-sink ldap $parts */
function runLdapSearchFn(string ...$parts): void {
}

runLdapSearchFn("a", "b", (string) $_GET["q"]);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'ldap'

class Searcher {
    /** @taint-sink ldap $parts */
    public function runLdapSearchMethod(string ...$parts): void {
    }
}

(new Searcher())->runLdapSearchMethod("a", "b", (string) $_GET["q"]);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'ldap'
