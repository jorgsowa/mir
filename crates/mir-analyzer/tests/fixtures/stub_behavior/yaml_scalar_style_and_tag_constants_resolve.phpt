===description===
The hand-written yaml stub declared only the encoding/break constants, so the
`YAML_*_SCALAR_STYLE` and `YAML_*_TAG` constants were reported undefined.
===file===
<?php

/** @return array<string, callable> */
function handlers(): array {
    return [YAML_STR_TAG => 'trim', YAML_MERGE_TAG => 'array_merge'];
}

function style(): int {
    return YAML_PLAIN_SCALAR_STYLE + YAML_FOLDED_SCALAR_STYLE;
}
