===description===
Each function declaration uses its own braced namespace, not the file's first namespace.
===config===
suppress=UnusedFunction
===file===
<?php
namespace First {
    function helper(): int {
        return 1;
    }
}

namespace Second {
    function helper(): string {
        return 1;
    }
}
===expect===
InvalidReturnType@10:8-10:17: Return type '1' is not compatible with declared 'string'
