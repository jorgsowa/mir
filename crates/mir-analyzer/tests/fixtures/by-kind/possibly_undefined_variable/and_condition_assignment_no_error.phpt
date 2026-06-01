===description===
no PossiblyUndefinedVariable when var is assigned inside && condition and used in if body
===file===
<?php
class Foo {
    public function test(object $e): void {
        // $inner assigned in RHS of &&; used in if body
        if (method_exists($e, 'getInner') &&
            ($inner = $e->getInner()) instanceof \RuntimeException) {
            throw $inner;
        }

        // $recaller assigned in RHS of &&; used in if body
        if (property_exists($e, 'recaller') && ! is_null($recaller = $e->getInner())) {
            echo $recaller;
        }
    }
}
===expect===
