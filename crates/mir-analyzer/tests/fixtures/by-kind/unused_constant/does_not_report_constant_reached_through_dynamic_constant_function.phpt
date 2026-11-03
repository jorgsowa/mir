===description===
`constant()` with a computed name inside the class may read any of its constants
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class Codes {
    private const NOT_FOUND = 404;
    private const GONE = 410;

    public function lookup(string $name): mixed {
        return constant(self::class . '::' . $name);
//             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception Error is thrown but not declared in @throws
    }
}

(new Codes())->lookup('GONE');
