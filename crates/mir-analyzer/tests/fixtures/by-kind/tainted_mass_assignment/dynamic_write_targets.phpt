===description===
A property, variable or session key whose name comes from the request lets the
attacker overwrite any of them.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayOffset errorLevel="suppress"/>
    <MixedPropertyFetch errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class User {
    public string $name = '';
    public bool $isAdmin = false;
}

function update(User $user): void {
    foreach ($_POST as $key => $value) {
        $user->$key = $value;
//      ^^^^^^^^^^^^^^^^^^^^ TaintedMassAssignment: Tainted property name — user input chooses what gets overwritten (mass assignment)
    }
}

function importVars(): void {
    $name = $_GET['name'];
    $$name = $_GET['value'];
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedMassAssignment: Tainted variable name — user input chooses what gets overwritten (mass assignment)
}

function remember(): void {
    $_SESSION[$_GET['k']] = $_GET['v'];
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedMassAssignment: Tainted $_SESSION key — user input chooses what gets overwritten (mass assignment)
}
