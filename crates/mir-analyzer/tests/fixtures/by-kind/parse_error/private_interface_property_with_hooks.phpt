===description===
Private interface property with hooks
===file===
<?php
interface A {
    private string $value { get; }
}
===expect===
ParseError@3:4-3:25: Parse error: Property in interface cannot be protected or private
