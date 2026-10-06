===description===
No visibility interface property with hooks
===file===
<?php
interface SomeInterface {
    string $value { get; }
//  ^^^^^^ ParseError: Parse error: expected modifier, found identifier
}
