===description===
No parent in attribute on class without parent
===config===
suppress=UnusedParam
===file===
<?php
#[Attribute]
class SomeAttr
{
    /** @param class-string $class */
    public function __construct(string $class) {}
}

#[SomeAttr(parent::class)]
class A {}

===expect===
ParentNotFound@9:11-9:17: Cannot use parent:: when current class has no parent
