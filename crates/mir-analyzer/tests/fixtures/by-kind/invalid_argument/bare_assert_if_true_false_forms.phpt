===description===
Bare (no-vendor-prefix) @assert-if-true/@assert-if-false forms were never
recognized — only the psalm-/phpstan-prefixed forms were, unlike sibling
tags @assert/@if-this-is, which both already have a bare form.
===file===
<?php
class C {
    /**
     * @param mixed $p
     * @assert-if-true int $p
     */
    public function isInt($p): bool {
        return is_int($p);
    }
}

function doWork(C $c, mixed $p): void {
    if ($c->isInt($p)) {
        strlen($p);
    }
}
===expect===
ArgumentTypeCoercion@14:15-14:17: Argument $string of strlen() expects 'string', got 'int' — coercion may fail at runtime
