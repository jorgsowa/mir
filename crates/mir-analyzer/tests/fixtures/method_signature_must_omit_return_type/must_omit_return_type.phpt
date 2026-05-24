===description===
mustOmitReturnType
===file===
<?php
class A
{
    public function __construct(): void
    {
    }
}
===expect===
MethodSignatureMustOmitReturnType
===ignore===
TODO
