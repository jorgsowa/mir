===description===
A nullable literal-int-union subject with an explicit null arm covering
every case is not flagged.
===file===
<?php
/** @param 1|2|null $n */
function label($n): string {
    return match ($n) {
        1 => "one",
        2 => "two",
        null => "none",
    };
}
===expect===
