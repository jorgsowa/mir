===description===
`@phpstan-ignore-next-line` sitting inside a multi-line docblock has the same
bug as bare `@phpstan-ignore`: it must skip past the docblock's own closing
`*/` to reach the declaration below, not target the closing line itself.
===file===
<?php
class C {
    /**
     * @phpstan-ignore-next-line
     */
    private function bar(): NoSuchClass { return new NoSuchClass(); }
}
===expect===
