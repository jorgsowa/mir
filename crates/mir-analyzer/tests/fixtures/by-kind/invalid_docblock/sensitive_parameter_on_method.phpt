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
    public function __construct(
        string $password
    ) {}
}

===expect===
InvalidAttribute@8:6-8:24: Attribute SensitiveParameter cannot be used on this target
