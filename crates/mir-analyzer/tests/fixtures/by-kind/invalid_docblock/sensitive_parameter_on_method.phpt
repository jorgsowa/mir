===description===
Sensitive parameter on method
===config===
suppress=UnusedParam
===file===
<?php

namespace SensitiveParameter;

use SensitiveParameter;

class HelloWorld {
    #[SensitiveParameter]
//    ^^^^^^^^^^^^^^^^^^ InvalidAttribute: Attribute SensitiveParameter cannot be used on this target
    public function __construct(
        string $password
    ) {}
}

===expect===
