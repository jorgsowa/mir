===description===
A method named as a callback, and a function with a `@taint-sink` parameter, receive the data the callback builtin hands them.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public function out(string $v): void {
        shell_exec($v);
    }

    public static function staticOut(string $v): void {
        shell_exec($v);
    }

    /** @taint-sink shell $v */
    public function annotated(string $v): void {}

    public function viaThis(): void {
        array_map([$this, 'out'], $_GET);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
    }

    public function clean(string $v): void {}
}

/** @taint-sink shell $v */
function run(string $v): void {}

function callers(A $a): void {
    array_map([$a, 'out'], $_GET);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
    array_map([new A(), 'out'], $_GET);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
    array_map([A::class, 'staticOut'], $_GET);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
    array_map('A::staticOut', $_GET);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
    array_map([$a, 'annotated'], $_GET);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
    call_user_func([$a, 'out'], $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
    call_user_func('run', $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
    array_map('run', $_GET);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}

function clean(A $a): void {
    array_map([$a, 'clean'], $_GET);
    call_user_func([$a, 'out'], 'ls');
}
