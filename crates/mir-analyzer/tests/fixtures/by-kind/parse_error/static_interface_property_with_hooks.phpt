===description===
Static interface property with hooks
===file===
<?php
interface A {
    public static string $value { get; }
}
===expect===
ParseError@3:5-3:41: Parse error: Cannot declare hooks for static property
