===description===
`scalar` and `numeric` are accepted by a union that spells out every type they stand for.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

function narrowed_scalar(mixed $data): array|string|int|float|bool|null {
    if (is_scalar($data)) {
        /** @mir-check $data is scalar */
        return $data;
    }
    return [];
}

/** @param numeric $n */
function numeric_parts($n): int|float|string {
    return $n;
}

/** @param scalar $s */
function unrelated($s): array {
    return $s;
//  ^^^^^^^^^^ InvalidReturnType: Return type 'scalar' is not compatible with declared 'array'
}
