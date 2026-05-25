===description===
Static interface property with hooks
===file===
<?php
interface A {
    public static string $value { get; }
}
===expect===
ParseError
===ignore===
TODO
