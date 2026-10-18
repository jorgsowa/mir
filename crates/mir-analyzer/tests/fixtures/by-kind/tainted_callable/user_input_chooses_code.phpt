===description===
User input used as a function, method or class name, or as a callback, lets
the attacker choose which code runs.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedFunctionCall errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Handler {
    public function run(): void {}
}

function dispatch(Handler $handler, array $items): void {
    $action = $_GET['action'];
    $action();
//  ^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    call_user_func($_GET['fn'], 1);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    array_map($_GET['fn'], $items);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    usort($items, $_GET['cmp']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    $method = $_GET['method'];
    $handler->$method();
//  ^^^^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    $class = $_GET['class'];
    new $class();
//  ^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
    $class::create();
//  ^^^^^^^^^^^^^^^^ TaintedCallable: Tainted callable — user input chooses the code that runs
}

function allowlisted(Handler $handler, array $items): void {
    $action = match ($_GET['action']) {
        'run' => 'trim',
        default => 'strtolower',
    };
    $action('x');
    call_user_func([$handler, 'run']);
    array_map('trim', $items);
    array_map(fn ($item) => $item, $_GET['items']);
    usort($items, fn ($a, $b) => $a <=> $b);
}
