===description===
noVisibilityInterfacePropertyWithHooks
===file===
<?php
interface SomeInterface {
    string $value { get; }
}
===expect===
ParseError
===ignore===
TODO
