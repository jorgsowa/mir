===description===
A nullable literal-string-union subject with an explicit null arm covering
every case is not flagged.
===file===
<?php
/** @param "a"|"b"|null $s */
function label($s): string {
    return match ($s) {
        "a" => "A",
        "b" => "B",
        null => "N",
    };
}
===expect===
