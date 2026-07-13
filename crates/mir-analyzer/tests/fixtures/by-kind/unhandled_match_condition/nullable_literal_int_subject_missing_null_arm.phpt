===description===
UnhandledMatchCondition fires for a nullable literal-int-union subject when
neither a `null` arm nor `default` is present, even if every non-null
literal is covered.
===file===
<?php
/** @param 1|2|null $n */
function label($n): string {
    return match ($n) {
        1 => "one",
        2 => "two",
    };
}
===expect===
UnhandledMatchCondition@4:11-7:5: Unhandled match condition: null
