===description===
No visibility interface property with hooks
===file===
<?php
interface SomeInterface {
    string $value { get; }
}
===expect===
ParseError@3:5-3:11: Parse error: expected modifier, found identifier
