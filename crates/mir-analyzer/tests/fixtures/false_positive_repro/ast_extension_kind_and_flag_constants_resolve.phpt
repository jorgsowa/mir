===description===
FP-C6: `ast\AST_*` node-kind constants and the separate `ast\flags\*`
namespace's flag constants must both resolve — same missing-stub root cause
as the parse_code/Node repro, but for the CONSTANTS section of the stub map.
===config===
suppress=UnusedParam
===file===
<?php

function isBinaryOp(int $kind): bool {
    return $kind === ast\AST_BINARY_OP;
}

function describeBinaryFlag(int $flag): string {
    return match ($flag) {
        ast\flags\BINARY_ADD => 'add',
        ast\flags\BINARY_SUB => 'sub',
        default => 'other',
    };
}
===expect===
