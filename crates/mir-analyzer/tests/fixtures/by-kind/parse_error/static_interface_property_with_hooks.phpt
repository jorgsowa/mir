===description===
Static interface property with hooks
===file===
<?php
interface A {
    public static string $value { get; }
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ParseError: Parse error: Cannot declare hooks for static property
}
