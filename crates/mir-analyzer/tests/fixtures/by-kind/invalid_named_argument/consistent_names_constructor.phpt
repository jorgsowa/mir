===description===
Consistent names constructor
===file===
<?php
/**
 * @consistent-constructor
 */
class A
{
    public function __construct(
        string $name,
        string $email,
    ) {}
}

class B extends A
{
    public function __construct(
        string $names,
        string $email,
    ) {}
}

===expect===
