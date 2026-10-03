===description===
Each function declaration uses its own braced namespace, not the file's first namespace.
===config===
<mir>
  <issueHandlers>
    <UnusedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//      ^^^^^^^^^ InvalidReturnType: Return type '1' is not compatible with declared 'string'
    }
}
===expect===
