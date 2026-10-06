===description===
`value-of<Cls::CONST>` in the global namespace enforces the constant's values the same way.
===file===
<?php
final class Kinds {
    public const array NAMES = ['a', 'b'];
}

class Repo {
    /** @param list<value-of<Kinds::NAMES>> $names */
    public function names(array $names): void { echo count($names); }
}

function caller(Repo $r): void {
    $valid = ['b'];
    /** @mir-check $valid is array{0: "b"} */
    $r->names($valid);

    $invalid = ['zzz'];
    /** @mir-check $invalid is array{0: "zzz"} */
    $r->names($invalid);
//            ^^^^^^^^ InvalidArgument: Argument $names of names() expects 'list<"a"|"b">', got 'array{0: "zzz"}'
}
