===description===
ast\flags\CLASS_ENUM and CLASS_READONLY resolve with php-ast's values
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
use const ast\flags\CLASS_ENUM;
use const ast\flags\CLASS_FINAL;
use const ast\flags\CLASS_READONLY;

function isEnumOrReadonly(int $flags): bool {
    return ($flags & CLASS_READONLY) > 0 || ($flags & CLASS_ENUM) > 0;
}

function values(): void {
    $enum = CLASS_ENUM;
    $readonly = CLASS_READONLY;
    $final = CLASS_FINAL;
    /** @mir-check $enum is 268435456 */
    /** @mir-check $readonly is 65536 */
    /** @mir-check $final is 32 */
}
===expect===
