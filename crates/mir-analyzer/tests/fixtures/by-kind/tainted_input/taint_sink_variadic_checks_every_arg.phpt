===description===
@taint-sink on a variadic parameter (`...$args`) only ever checked the
FIRST variadic call-site argument's position — a later variadic argument
silently bypassed the check entirely, in both the plain-function and
method-call taint-sink paths.
===config===
suppress=MixedArrayAccess,UnusedParam,MissingConstructor
===file===
<?php
/** @taint-sink ldap $parts */
function runLdapSearchFn(string ...$parts): void {
}

runLdapSearchFn("a", "b", (string) $_GET["q"]);

class Searcher {
    /** @taint-sink ldap $parts */
    public function runLdapSearchMethod(string ...$parts): void {
    }
}

(new Searcher())->runLdapSearchMethod("a", "b", (string) $_GET["q"]);
===expect===
TaintedInput@6:0-6:46: Tainted input reaching sink 'ldap'
TaintedInput@14:0-14:68: Tainted input reaching sink 'ldap'
