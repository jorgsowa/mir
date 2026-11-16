===description===
Rejecting a leading dash says nothing about which property the name selects.
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
final class User {
    public bool $isAdmin = false;
}

function update(User $user): void {
    $key = $_GET['k'];
    if (str_starts_with($key, '-')) {
        return;
    }
    $user->$key = true;
//  ^^^^^^^^^^^^^^^^^^ TaintedMassAssignment: Tainted property name — user input chooses what gets overwritten (mass assignment)
}
