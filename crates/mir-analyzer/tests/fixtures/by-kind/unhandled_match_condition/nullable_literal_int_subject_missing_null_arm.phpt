===description===
UnhandledMatchCondition fires for a nullable literal-int-union subject when
neither a `null` arm nor `default` is present, even if every non-null
literal is covered.
===file===
<?php
/** @param 1|2|null $n */
function label($n): string {
    return match ($n) {
//         ^ +3:5 UnhandledMatchCondition: Unhandled match condition: null
        1 => "one",
        2 => "two",
    };
}
