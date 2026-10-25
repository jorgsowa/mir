===description===
Sensitive parameter on method
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

namespace SensitiveParameter;

use SensitiveParameter;

class HelloWorld {
    #[SensitiveParameter]
//    ^^^^^^^^^^^^^^^^^^ CompileError: Attribute "SensitiveParameter" cannot target method (allowed targets: parameter) [attribute-target-mismatch]
    public function __construct(
        string $password
    ) {}
}
