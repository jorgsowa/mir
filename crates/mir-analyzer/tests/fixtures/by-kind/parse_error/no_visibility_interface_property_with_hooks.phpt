===description===
No visibility interface property with hooks
===file===
<?php
interface SomeInterface {
    string $value { get; }
}
===expect===
ParseError
===ignore===
TODO
