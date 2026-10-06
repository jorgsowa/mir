===description===
A variable-variable assignment (`${"$key"} = ...`) defines variables whose names
are not statically known, so later reads must not be reported as UndefinedVariable.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
function run(array $opts): void {
    foreach (['a', 'b'] as $key) {
        ${"$key"} = $opts[$key] ?? null;
    }
    // FP expected: UndefinedVariable $a (variable-variables not tracked)
    echo $a;
}
