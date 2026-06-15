===description===
No visibility interface property with hooks
===file===
<?php
interface SomeInterface {
    string $value { get; }
}
===expect===
ParseError@3:4-3:10: Parse error: expected modifier, found identifier
