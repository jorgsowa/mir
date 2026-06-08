===description===
Sensitive parameter on method
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
InvalidAttribute@8:7-8:25: Attribute SensitiveParameter cannot be used on this target
