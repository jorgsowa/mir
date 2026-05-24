===description===
interfacePropertyWithHooksBeforePhp84
===file===
<?php
interface A {
    public string $value { get; }
}
===expect===
ParseError
===ignore===
TODO
