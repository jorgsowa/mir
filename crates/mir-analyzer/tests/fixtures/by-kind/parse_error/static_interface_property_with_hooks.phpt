===description===
Static interface property with hooks
===file===
<?php
interface A {
    public static string $value { get; }
}
===expect===
ParseError@3:4-3:40: Parse error: Cannot declare hooks for static property
