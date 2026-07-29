===description===
Negative control for the ast/ast.php stub fix: vendoring the real stub file
must not turn `ast\*` into a wildcard-resolved namespace — a name that isn't
actually part of the extension must still be flagged undefined.
===file===
<?php
function inspect(string $code): void {
    ast\parse_codee($code, 90);
}
===expect===
UndefinedFunction@3:4-3:30: Function ast\parse_codee() is not defined
