===description===
Must omit return type
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
