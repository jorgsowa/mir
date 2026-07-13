===description===
UnhandledMatchCondition fires for a nullable literal-string-union subject
when neither a `null` arm nor `default` is present, even if every non-null
literal is covered.
===file===
<?php
/** @param "a"|"b"|null $s */
function label($s): string {
    return match ($s) {
        "a" => "A",
        "b" => "B",
    };
}
===expect===
UnhandledMatchCondition@4:11-7:5: Unhandled match condition: null
