===description===
`(object)` turns an array or scalar into a stdClass and leaves an object unchanged.
===file===
<?php
/** @param array<string, mixed> $output */
function toMap(array $output, bool $asObject): array|\stdClass {
    if ($asObject) {
        return (object) $output;
    }
    return $output;
}

function keep(\ArrayObject $o): \ArrayObject {
    return (object) $o;
}
