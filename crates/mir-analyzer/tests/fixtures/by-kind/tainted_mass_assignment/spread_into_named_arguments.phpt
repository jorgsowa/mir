===description===
Since PHP 8.0 string keys of a spread array bind named parameters, so spreading
request data lets the attacker choose which parameters get set.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function createUser(string $name, string $email = '', bool $isAdmin = false): void {}

final class Account {
    public function __construct(string $name, bool $isAdmin = false) {}
}

function viaFunction(): void {
    createUser(...$_POST);
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedMassAssignment: Tainted array keys spread into named arguments — user input chooses what gets overwritten (mass assignment)
}

function viaVariable(): void {
    $data = $_POST;
    new Account(...$data);
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedMassAssignment: Tainted array keys spread into named arguments — user input chooses what gets overwritten (mass assignment)
}

function viaCallUserFuncArray(): void {
    call_user_func_array('createUser', $_POST);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedMassAssignment: Tainted array keys spread into named arguments — user input chooses what gets overwritten (mass assignment)
}

function viaDynamicClass(string $class): void {
    $data = $_POST;
    new $class(...$data);
//  ^^^^^^^^^^^^^^^^^^^^ TaintedMassAssignment: Tainted array keys spread into named arguments — user input chooses what gets overwritten (mass assignment)
}
