===description===
`getenv()` with a name that can't be null returns `string|false`; with no name
or `null` it returns the whole environment.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedForeachValue errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param array<string, string> $values */
function f(string $name, array $values, ?string $maybe): void {
    $a = (string) getenv($name);
    foreach ($values as $key => $value) {
        $b = (string) getenv($key);
        $c = getenv($key, true);
        /** @trace $c */
        echo 1;
//      ^^^^^^^ Trace: Type of $c is string|false
    }
    $all = getenv();
    /** @trace $all */
    echo 1;
//  ^^^^^^^ Trace: Type of $all is array<string, string>|false
    $either = getenv($maybe);
    /** @trace $either */
    echo 1;
//  ^^^^^^^ Trace: Type of $either is string|array|false
}
