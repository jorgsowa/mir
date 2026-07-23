===description===
@taint-sink with a kind string that isn't one of the recognized ones
(llm_prompt/html/sql/shell) falls back to the generic TaintedInput issue,
naming the kind, instead of silently doing nothing.
===config===
suppress=MixedArrayAccess,UnusedParam
===file===
<?php
/** @taint-sink ldap $filter */
function runLdapSearch(string $filter): void {
}

runLdapSearch((string) $_GET["q"]);
===expect===
TaintedInput@6:0-6:34: Tainted input reaching sink 'ldap'
