===description===
Environment helpers reading `$_SERVER[$key]` feed file, shell and callable
sinks only when the caller passes request data as the key; `array_intersect_key`
takes values from its first array only.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function env(string $key): string {
    return $_SERVER[$key] ?? '';
}

function sinks(): void {
    file_get_contents(env('HOME') . '/x');
    shell_exec('cd ' . env('PWD'));
    file_get_contents(env($_GET['k']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
}

/** @param array<string, string> $env */
function keyOnlyArrays(array $env): void {
    $kept = array_intersect_key($env, $_SERVER);
    $dropped = array_diff_key($env, $_SERVER);
    shell_exec(implode(' ', $kept));
    shell_exec(implode(' ', $dropped));
    $values = array_intersect_key($_SERVER, $env);
    shell_exec(implode(' ', $values));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}
