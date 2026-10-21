===description===
Protected interface property with hooks
===file===
<?php
interface A {
    protected string $value { get; }
}
===expect===
ParseError@3:4-3:27: Parse error: Property in interface cannot be protected or private
