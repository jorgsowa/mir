===description===
@taint-sink with a kind string that isn't one of the recognized ones
(html, sql, shell, ldap, ...) falls back to the generic TaintedInput issue,
naming the kind, instead of silently doing nothing.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @taint-sink graphql $filter */
function runGraphqlQuery(string $filter): void {
}

runGraphqlQuery((string) $_GET["q"]);
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'graphql'
