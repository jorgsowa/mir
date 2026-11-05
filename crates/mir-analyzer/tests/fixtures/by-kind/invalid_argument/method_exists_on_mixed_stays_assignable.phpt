===description===
A value only checked with `method_exists` can still be passed where a typed parameter is expected.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Driver {}

function take(Driver $d): Driver { return $d; }

function after_the_check(mixed $n): void {
    if (method_exists($n, 'after')) {
        $n->after();
    }
    take($n);
}

function in_the_false_branch(mixed $n): void {
    if (!method_exists($n, 'after')) {
        take($n);
    }
}
