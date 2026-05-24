===description===
protectedInterfacePropertyWithHooks
===file===
<?php
interface A {
    protected string $value { get; }
}
===expect===
ParseError
===ignore===
TODO
